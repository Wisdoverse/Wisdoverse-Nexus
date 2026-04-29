import { useEffect } from 'react'
import { ActivityIndicator, FlatList, Pressable, StyleSheet, Text, View } from 'react-native'

import { useRoomsStore } from './roomsStore'
import type { RootStackNavigationProp } from '../../app/navigation/types'

interface Props {
  navigation: RootStackNavigationProp
}

export function RoomListScreen({ navigation }: Props) {
  const { rooms, loading, error, fetchRooms } = useRoomsStore()

  useEffect(() => {
    void fetchRooms()
  }, [fetchRooms])

  return (
    <View style={styles.container}>
      {loading ? <ActivityIndicator size="large" color="#0b5fff" /> : null}
      {error ? <Text style={styles.error}>{error}</Text> : null}
      <FlatList
        data={rooms}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.listContent}
        renderItem={({ item }) => (
          <Pressable
            style={styles.card}
            onPress={() =>
              navigation.navigate('MessageStream', {
                roomId: item.id,
                roomName: item.name,
              })
            }
          >
            <Text style={styles.name}>{item.name}</Text>
            <Text style={styles.topic}>{item.topic || 'No topic'}</Text>
          </Pressable>
        )}
      />
    </View>
  )
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F4F7FB',
    padding: 16,
  },
  listContent: {
    gap: 10,
    paddingBottom: 16,
  },
  card: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 14,
    borderWidth: 1,
    borderColor: '#E2E8F0',
  },
  name: {
    fontSize: 16,
    fontWeight: '700',
    color: '#111827',
  },
  topic: {
    marginTop: 6,
    fontSize: 13,
    color: '#64748B',
  },
  error: {
    color: '#B42318',
    marginBottom: 10,
  },
})
