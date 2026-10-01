"""Opt-in real-model smoke against an already configured evaluation gateway.

Required runtime variables: NEXIS_REAL_PROVIDER_SMOKE=true,
NEXIS_REAL_GATEWAY_URL, NEXIS_REAL_SMOKE_TOKEN, NEXIS_REAL_MODEL_NAME,
NEXIS_REAL_PRICE_DATE (ISO date), NEXIS_REAL_INPUT_USD_PER_MILLION,
NEXIS_REAL_OUTPUT_USD_PER_MILLION, NEXIS_REAL_COST_BUDGET_USD.
Credentials must be injected; none are written to the evidence. One synthetic
room and one bounded model invocation are created only after explicit opt-in.
This smoke is not the 50-case quality/pilot acceptance.
"""
from __future__ import annotations
import asyncio
from datetime import date
import json
import math
import os
from pathlib import Path
from urllib.parse import urlsplit
from nexis import CreateRoomData, NexisClient


async def main():
    if os.environ.get('NEXIS_REAL_PROVIDER_SMOKE') != 'true':
        print('SKIP real-provider smoke: explicit runtime opt-in is required')
        return
    url = os.environ['NEXIS_REAL_GATEWAY_URL']
    endpoint = urlsplit(url)
    if endpoint.scheme != 'https' and not (endpoint.scheme == 'http' and endpoint.hostname in {'127.0.0.1', 'localhost', '::1'}):
        raise ValueError('Gateway requires HTTPS or loopback HTTP')
    rates = [float(os.environ[name]) for name in ('NEXIS_REAL_INPUT_USD_PER_MILLION', 'NEXIS_REAL_OUTPUT_USD_PER_MILLION', 'NEXIS_REAL_COST_BUDGET_USD')]
    if any(not math.isfinite(value) or value < 0 for value in rates) or rates[2] <= 0:
        raise ValueError('Finite nonnegative prices and a positive cost budget are required')
    price_date = date.fromisoformat(os.environ['NEXIS_REAL_PRICE_DATE']).isoformat()
    # Conservative input byte/token upper bound includes fixed prompt overhead.
    upper_cost = (40960 * rates[0] + 256 * rates[1]) / 1_000_000
    if upper_cost > rates[2]:
        raise ValueError('Worst-case configured usage exceeds the cost budget')
    async with NexisClient(url) as client:
        await client.authenticate(os.environ['NEXIS_REAL_SMOKE_TOKEN'])
        room = await client.create_room(CreateRoomData('real-provider-synthetic-smoke'))
        source = await client.send_message(room.id, 'Synthetic decision: use a blue cover. No action is approved.')
        try:
            capabilities = await client.agent_capabilities(room.id)
            if not capabilities['enabled']:
                raise RuntimeError('Configured assistant is disabled')
            run = await client.invoke_agent(room.id, 'State the synthetic cover decision. Cite its source message ID; do not claim any action was performed.', source_message_ids=[source.id], max_output_tokens=256, deadline_ms=60000)
            async for _ in client.observe_agent_run(room.id, run['id']):
                pass
            result = await client.get_agent_run(room.id, run['id'])
            usage = result['usage']
            if result['status'] != 'completed' or usage['inputTokens'] is None or usage['outputTokens'] is None:
                raise RuntimeError('Real provider did not complete with reported usage')
            cost = (usage['inputTokens'] * rates[0] + usage['outputTokens'] * rates[1]) / 1_000_000
            grounded = 'blue' in result['answer'].lower() and source.id in result['answer']
            report = {'schema_version': 1, 'provider': 'configured real OpenAI-compatible model', 'declared_model': os.environ['NEXIS_REAL_MODEL_NAME'], 'pricing_date': price_date, 'input_usd_per_million': rates[0], 'output_usd_per_million': rates[1], 'usage': usage, 'latency_ms': result['latencyMs'], 'estimated_cost_usd': cost, 'budget_usd': rates[2], 'synthetic_grounding_passed': grounded, 'passed': grounded and cost <= rates[2], 'limitations': 'One opted-in synthetic request; no pilot, injection resistance or broad quality qualification.'}
            output = Path('artifacts/real-provider')
            output.mkdir(parents=True, exist_ok=True)
            (output / 'summary.json').write_text(json.dumps(report, indent=2) + '\n')
            if not report['passed']:
                raise RuntimeError('Grounding/cost smoke failed; inspect the redacted summary')
            print('PASS real-provider synthetic smoke; artifacts/real-provider/summary.json')
        finally:
            await client.delete_room(room.id)


if __name__ == '__main__':
    asyncio.run(main())
