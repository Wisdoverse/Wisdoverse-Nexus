const assert = require('node:assert/strict');
const { NexisClient } = require('../dist');
(async () => {
  const client = new NexisClient({ baseUrl: process.env.NEXIS_SMOKE_URL });
  try {
    await client.authenticate(process.env.NEXIS_SMOKE_TOKEN);
    const shared = await client.getAgentRun(process.env.NEXIS_AGENT_SHARED_ROOM, process.env.NEXIS_AGENT_SHARED_RUN);
    assert.equal(shared.answer, 'Synthetic response 🦀');
    assert.equal(shared.status, 'completed');
    const room = await client.createRoom({ name: 'typescript-agent-smoke' });
    const source = await client.sendMessage(room.id, 'SDK source');
    assert.equal((await client.agentCapabilities(room.id)).enabled, true);
    const options = { clientRunId: 'typescript-stable', sourceMessageIds: [source.id] };
    const run = await client.invokeAgent(room.id, 'Summarize', options);
    const events = [];
    for await (const event of client.observeAgentRun(room.id, run.id, { timeoutMs: 5000, pollMs: 20 })) events.push(event);
    assert.equal(events.at(-1).kind, 'completed');
    const result = await client.getAgentRun(room.id, run.id);
    assert.equal(result.answer, shared.answer);
    assert.deepEqual(result.sourceMessageIds, [source.id]);
    assert.equal(result.usage.outputTokens, 8);
    assert.equal((await client.invokeAgent(room.id, 'Summarize', options)).id, run.id);
    assert.equal((await client.cancelAgentRun(room.id, run.id)).status, 'completed');
    assert.deepEqual((await client.getAgentEvents(room.id, run.id, events.at(-1).sequence)).events, []);
    console.log('PASS TypeScript SDK observes the same Python run and its own ordered/idempotent run');
  } finally { client.logout(); }
})().catch(error => { console.error(error.name, error.response?.status ?? 'SDK assertion failed'); process.exitCode = 1; });
