import type { NativeStackNavigationProp, NativeStackScreenProps } from '@react-navigation/native-stack'

export type RootStackParamList = {
  MainTabs: undefined
  MessageStream: {
    roomId: string
    roomName: string
  }
}

export type MainTabParamList = {
  Rooms: undefined
  Search: undefined
}

export type RootStackNavigationProp = NativeStackNavigationProp<RootStackParamList>
export type MessageStreamRouteProp = NativeStackScreenProps<RootStackParamList, 'MessageStream'>['route']
