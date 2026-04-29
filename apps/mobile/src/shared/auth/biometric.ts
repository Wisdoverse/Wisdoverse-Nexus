import * as LocalAuthentication from 'expo-local-authentication'

export interface BiometricSupport {
  available: boolean
  enrolled: boolean
}

export async function getBiometricSupport(): Promise<BiometricSupport> {
  const [hardware, enrolled] = await Promise.all([
    LocalAuthentication.hasHardwareAsync(),
    LocalAuthentication.isEnrolledAsync(),
  ])

  return {
    available: hardware,
    enrolled,
  }
}

export async function authenticateWithBiometrics(reason = 'Authenticate to continue'): Promise<boolean> {
  const support = await getBiometricSupport()
  if (!support.available || !support.enrolled) {
    return false
  }

  const result = await LocalAuthentication.authenticateAsync({
    promptMessage: reason,
    fallbackLabel: 'Use passcode',
  })

  return result.success
}
