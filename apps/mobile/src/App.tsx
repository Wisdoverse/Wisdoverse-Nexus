import { StatusBar } from 'expo-status-bar'

import { RootNavigator } from './app/navigation/RootNavigator'
import { AppProviders } from './app/providers/AppProviders'

export default function App() {
  return (
    <AppProviders>
      <StatusBar style="light" />
      <RootNavigator />
    </AppProviders>
  )
}
