import { useState } from 'react'
import { ActivityIndicator, FlatList, Pressable, StyleSheet, Text, TextInput, View } from 'react-native'

import { useSearchStore } from './searchStore'

export function SearchScreen() {
  const [query, setQuery] = useState('')
  const { results, loading, error, search } = useSearchStore()

  const onSearch = async () => {
    const q = query.trim()
    if (!q) {
      return
    }
    await search(q)
  }

  return (
    <View style={styles.container}>
      <View style={styles.searchRow}>
        <TextInput
          value={query}
          onChangeText={setQuery}
          placeholder="Search messages / rooms"
          placeholderTextColor="#94A3B8"
          style={styles.input}
        />
        <Pressable onPress={onSearch} style={styles.button}>
          <Text style={styles.buttonText}>Go</Text>
        </Pressable>
      </View>

      {loading ? <ActivityIndicator size="small" color="#0b5fff" /> : null}
      {error ? <Text style={styles.error}>{error}</Text> : null}

      <FlatList
        data={results}
        keyExtractor={(item) => `${item.type}-${item.id}`}
        contentContainerStyle={styles.list}
        renderItem={({ item }) => (
          <View style={styles.resultCard}>
            <Text style={styles.title}>{item.title}</Text>
            <Text style={styles.meta}>{item.type}</Text>
            {item.snippet ? <Text style={styles.snippet}>{item.snippet}</Text> : null}
          </View>
        )}
      />
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
  searchRow: {
    flexDirection: 'row',
    gap: 8,
  },
  input: {
    flex: 1,
    borderWidth: 1,
    borderColor: '#CBD5E1',
    borderRadius: 10,
    backgroundColor: '#FFFFFF',
    paddingHorizontal: 12,
    paddingVertical: 10,
    color: '#111827',
  },
  button: {
    borderRadius: 10,
    backgroundColor: '#0b5fff',
    paddingHorizontal: 14,
    justifyContent: 'center',
  },
  buttonText: {
    color: '#FFFFFF',
    fontWeight: '700',
  },
  list: {
    gap: 8,
    paddingBottom: 20,
  },
  resultCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 10,
    borderWidth: 1,
    borderColor: '#E2E8F0',
    padding: 12,
  },
  title: {
    fontSize: 15,
    fontWeight: '700',
    color: '#111827',
  },
  meta: {
    fontSize: 12,
    color: '#64748B',
    marginTop: 2,
  },
  snippet: {
    fontSize: 13,
    color: '#334155',
    marginTop: 6,
  },
  error: {
    color: '#B42318',
  },
})
