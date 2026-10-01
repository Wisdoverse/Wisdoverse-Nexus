import { useEffect } from 'react'
import { ActivityIndicator, Text, View } from 'react-native'
import { useAuthStore, TokenLoginScreen } from '../../features/auth'
import { NavigationContainer } from '@react-navigation/native'
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs'
import { createNativeStackNavigator } from '@react-navigation/native-stack'

import { RoomListScreen } from '../../features/rooms'
import { SearchScreen } from '../../features/search'
import { MessageStreamScreen } from '../../features/messages'
import type { MainTabParamList, RootStackParamList } from '../../shared/navigation'

const Stack = createNativeStackNavigator<RootStackParamList>()
const Tab = createBottomTabNavigator<MainTabParamList>()

function MainTabs() {
  return (
    <Tab.Navigator
      screenOptions={{
        headerStyle: { backgroundColor: '#0A0F1A' },
        headerTintColor: '#F8FAFC',
        tabBarStyle: { backgroundColor: '#FFFFFF' },
        tabBarActiveTintColor: '#0b5fff',
      }}
    >
      <Tab.Screen name="Rooms" component={RoomListScreen} options={{ title: 'Rooms' }} />
      <Tab.Screen name="Search" component={SearchScreen} options={{ title: 'Search' }} />
    </Tab.Navigator>
  )
}

export function RootNavigator() {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  const isHydrated = useAuthStore((state) => state.isHydrated)
  const restoreSession = useAuthStore((state) => state.restoreSession)
  useEffect(() => { void restoreSession() }, [restoreSession])
  if (!isHydrated) return <View style={{ flex: 1, justifyContent: 'center', alignItems: 'center', gap: 12 }}>
    <ActivityIndicator accessibilityLabel="Restoring session" />
    <Text accessibilityLiveRegion="polite">Restoring your session...</Text>
  </View>
  if (!isAuthenticated) return <TokenLoginScreen />
  return (
    <NavigationContainer>
      <Stack.Navigator
        screenOptions={{
          headerStyle: { backgroundColor: '#0A0F1A' },
          headerTintColor: '#F8FAFC',
          contentStyle: { backgroundColor: '#F8FAFC' },
        }}
      >
        <Stack.Screen name="MainTabs" component={MainTabs} options={{ headerShown: false }} />
        <Stack.Screen
          name="MessageStream"
          component={MessageStreamScreen}
          options={({ route }) => ({ title: route.params.roomName })}
        />
      </Stack.Navigator>
    </NavigationContainer>
  )
}
