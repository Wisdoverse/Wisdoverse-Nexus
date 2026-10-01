import { useState } from 'react'
import { Pressable, Text, TextInput, View } from 'react-native'
import { authApi } from '../../shared/api/authApi'
import { useAuthStore } from './authStore'

export function TokenLoginScreen() {
  const [token, setToken] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const signIn = async () => {
    setBusy(true)
    setError('')
    try {
      const session = await authApi.authenticate(token.trim())
      await useAuthStore.getState().login(session)
    } catch { setError('Sign in failed. Check your token and gateway connection.') }
    finally { setBusy(false) }
  }
  return <View style={{ flex: 1, padding: 24, justifyContent: 'center', gap: 16 }}>
    <Text style={{ fontSize: 24 }}>Sign in to Wisdoverse Nexus</Text>
    <Text>Enter the access token issued by your team.</Text>
    <TextInput accessibilityLabel="Access token" secureTextEntry autoCapitalize="none" autoCorrect={false}
      value={token} onChangeText={setToken} placeholder="Access token"
      style={{ borderWidth: 1, padding: 12, borderRadius: 8 }} />
    {error ? <Text accessibilityRole="alert">{error}</Text> : null}
    <Pressable accessibilityRole="button" disabled={busy || !token.trim()} onPress={signIn}>
      <Text>{busy ? 'Signing in...' : 'Sign in'}</Text>
    </Pressable>
  </View>
}
