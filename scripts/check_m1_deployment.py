"""Render-only qualification of the single-instance M1 deployment profile."""
from pathlib import Path
import subprocess

import yaml

ROOT = Path(__file__).resolve().parents[1]
CHART = ROOT / "deploy/helm/nexis-gateway"
HELM_IMAGE = "alpine/helm@sha256:aef9b56f64e866207d9591d0abd8f6d767b36aadd12edf68f8a719716d9d29c9"
FIXTURE_DIGEST = "sha256:" + "0" * 64


def helm(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["docker", "run", "--rm", "--network", "none", "--entrypoint", "helm",
         "-v", f"{CHART}:/chart:ro", HELM_IMAGE, *args],
        text=True, capture_output=True, timeout=120, check=False,
    )


def require(condition: bool, description: str) -> None:
    if not condition:
        raise AssertionError(description)


def validate(resources: list[dict]) -> dict:
    deployments = [resource for resource in resources if resource["kind"] == "Deployment"]
    require(len(deployments) == 1, "exactly one deployment is required")
    require(not any(resource["kind"] == "HorizontalPodAutoscaler" for resource in resources), "M1 must not autoscale")
    deployment = deployments[0]
    require(deployment["spec"]["replicas"] == 1, "M1 must have one gateway instance")
    require(deployment["spec"]["strategy"]["type"] == "Recreate", "upgrades must not split process-local state")
    pod = deployment["spec"]["template"]["spec"]
    require(pod["terminationGracePeriodSeconds"] >= 30, "HTTP drain needs a termination grace period")
    require(pod["securityContext"]["runAsNonRoot"], "gateway must run without root")
    container = pod["containers"][0]
    security = container["securityContext"]
    require(security["allowPrivilegeEscalation"] is False, "privilege escalation must be disabled")
    require(security["readOnlyRootFilesystem"], "M1 needs no writable root filesystem")
    require("ALL" in security["capabilities"]["drop"], "container capabilities must be dropped")
    for probe in ("startupProbe", "readinessProbe", "livenessProbe"):
        require(container[probe]["httpGet"]["path"] == "/health", f"{probe} must use the tested health endpoint")
    for budget in ("requests", "limits"):
        require(all(key in container["resources"][budget] for key in ("cpu", "memory")), "CPU and memory budgets are required")
    jwt = next(env for env in container["env"] if env["name"] == "JWT_SECRET")
    require("value" not in jwt, "JWT key material must not be embedded in manifests")
    require(bool(jwt["valueFrom"]["secretKeyRef"]["name"]), "JWT must reference an existing Secret")
    return container


def main() -> None:
    lint = helm("lint", "/chart", "--strict", "--set", "secrets.existingSecret=synthetic-auth")
    require(lint.returncode == 0, f"Helm lint failed: {lint.stderr}")
    rendered = helm("template", "m1-fixture", "/chart", "--set", "secrets.existingSecret=synthetic-auth",
                    "--set", f"image.digest={FIXTURE_DIGEST}")
    require(rendered.returncode == 0, f"Helm render failed: {rendered.stderr}")
    resources = [resource for resource in yaml.safe_load_all(rendered.stdout) if resource]
    container = validate(resources)
    require(container["image"].endswith("@" + FIXTURE_DIGEST), "image digest must be preserved")
    for options, expected in [([], "secrets.existingSecret"),
                              (["--set", "secrets.existingSecret=synthetic-auth", "--set", "replicaCount=2"], "M1 requires"),
                              (["--set", "secrets.existingSecret=synthetic-auth", "--set", "autoscaling.enabled=true"], "M1 requires")]:
        result = helm("template", "m1-fixture", "/chart", *options)
        require(result.returncode != 0 and expected in result.stderr, "unsafe Helm configuration must fail explicitly")
    validate([resource for resource in yaml.safe_load_all((ROOT / "k8s/gateway.yaml").read_text()) if resource])
    compose = yaml.safe_load((ROOT / "deploy/docker-compose.yml").read_text())
    require(any(value.startswith("JWT_SECRET=${JWT_SECRET:?") for value in compose["services"]["nexis-gateway"]["environment"]),
            "development Compose must inject the correct signing-key variable")
    print("M1 deployment checks passed: Helm lint/render, digest/Secret injection, rejected unsafe profiles, raw Kubernetes and Compose.")


if __name__ == "__main__":
    main()
