import { NavigationContainer } from '@react-navigation/native'
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs'
import { createNativeStackNavigator } from '@react-navigation/native-stack'

import { RoomListScreen } from '../../features/rooms/RoomListScreen'
import { SearchScreen } from '../../features/search/SearchScreen'
import { MessageStreamScreen } from '../../features/messages/MessageStreamScreen'
import type { MainTabParamList, RootStackParamList } from './types'

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
