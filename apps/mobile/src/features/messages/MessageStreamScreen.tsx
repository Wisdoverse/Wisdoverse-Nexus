import { useEffect, useState } from 'react'
import { ActivityIndicator, FlatList, Pressable, StyleSheet, Text, TextInput, View } from 'react-native'

import { useMessagesStore } from './messagesStore'
import type { MessageStreamRouteProp } from '../../app/navigation/types'

interface Props {
  route: MessageStreamRouteProp
}

export function MessageStreamScreen({ route }: Props) {
  const { roomId, roomName } = route.params
  const { messages, loading, sending, error, fetchMessages, sendMessage } = useMessagesStore()
  const [draft, setDraft] = useState('')

  useEffect(() => {
    void fetchMessages(roomId)
  }, [fetchMessages, roomId])

  const onSend = async () => {
    const text = draft.trim()
    if (!text) {
      return
    }
    setDraft('')
    await sendMessage(roomId, text)
  }

  return (
    <View style={styles.container}>
      <Text style={styles.title}>{roomName}</Text>
      {loading ? <ActivityIndicator size="small" color="#0b5fff" /> : null}
      {error ? <Text style={styles.error}>{error}</Text> : null}

      <FlatList
        data={messages}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.list}
        renderItem={({ item }) => (
          <View style={styles.messageCard}>
            <Text style={styles.sender}>{item.sender}</Text>
            <Text style={styles.text}>{item.text}</Text>
          </View>
        )}
      />

      <View style={styles.composer}>
        <TextInput
          value={draft}
          onChangeText={setDraft}
          style={styles.input}
          placeholder="Write a message"
          placeholderTextColor="#94A3B8"
        />
        <Pressable onPress={onSend} disabled={sending} style={styles.sendButton}>
          <Text style={styles.sendText}>{sending ? '...' : 'Send'}</Text>
        </Pressable>
      </View>
    </View>
  )
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F8FAFC',
    padding: 16,
    gap: 12,
  },
  title: {
    fontSize: 18,
    fontWeight: '700',
    color: '#0F172A',
  },
  list: {
    gap: 8,
    paddingBottom: 16,
  },
  messageCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 10,
    borderWidth: 1,
    borderColor: '#E2E8F0',
    padding: 12,
  },
  sender: {
    fontSize: 12,
    color: '#475569',
    marginBottom: 2,
  },
  text: {
    fontSize: 15,
    color: '#111827',
  },
  composer: {
    flexDirection: 'row',
    gap: 8,
    alignItems: 'center',
  },
  input: {
    flex: 1,
    backgroundColor: '#FFFFFF',
    borderWidth: 1,
    borderColor: '#CBD5E1',
    borderRadius: 10,
    paddingHorizontal: 12,
    paddingVertical: 10,
    color: '#111827',
  },
  sendButton: {
    backgroundColor: '#0b5fff',
    paddingHorizontal: 14,
    paddingVertical: 10,
    borderRadius: 10,
  },
  sendText: {
    color: '#FFFFFF',
    fontWeight: '700',
  },
  error: {
    color: '#B42318',
  },
})
