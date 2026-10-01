const assert = require('node:assert/strict');
const { NexisClient } = require('../dist');
const WebSocket = require('ws');
const baseUrl = process.env.NEXIS_SMOKE_URL;
const token = process.env.NEXIS_SMOKE_TOKEN;
const peerToken = process.env.NEXIS_SMOKE_PEER_TOKEN;
const waitUntil = async (condition) => {
  const deadline = Date.now() + 8000;
  while (!condition()) {
    if (Date.now() > deadline) throw new Error('Timed out waiting for gateway event');
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
};
(async () => {
  const client = new NexisClient({ baseUrl });
  const peer = new NexisClient({ baseUrl });
  const events = [];
  let replacement;
  try {
    await assert.rejects(client.authenticate('invalid-token'));
    await assert.rejects(client.authenticate(process.env.NEXIS_SMOKE_EXPIRED_TOKEN));
    const session = await client.authenticate(token);
    assert.equal(session.memberId, 'm1-owner');
    await peer.authenticate(peerToken);
    const room = await client.createRoom({ name: 'typescript-smoke', topic: 'synthetic' });
    assert((await client.listRooms()).some((item) => item.id === room.id));
    assert(!(await peer.listRooms()).some((item) => item.id === room.id));
    await assert.rejects(peer.getRoom(room.id), (error) => error.response.status === 403);
    await client.inviteMember(room.id, 'm1-peer');
    client.on('new_message', (event) => events.push(event));
    const manager = client.connect(room.id);
    let connections = 0;
    manager.onConnectionChange((state) => { if (state === 'connected') connections++; });
    await waitUntil(() => manager.isConnected());
    const message = await peer.sendMessage(room.id, 'shared event');
    await waitUntil(() => events.some((event) => event.message_id === message.id));
    assert.equal(events[0].content, message.text);
    const retryKey = 'typescript-stable-retry';
    const accepted = await client.sendMessage(room.id, 'once', undefined, retryKey);
    const retried = await client.sendMessage(room.id, 'once', undefined, retryKey);
    assert.equal(retried.id, accepted.id);
    await assert.rejects(client.sendMessage(room.id, 'changed', undefined, retryKey), (error) => error.response.status === 409);
    assert.equal((await client.getMessages(room.id)).filter((item) => item.id === accepted.id).length, 1);
    // Authenticate a replacement socket for this member to force a genuine transport disconnect.
    replacement = new WebSocket(baseUrl.replace(/^http/, 'ws') + '/ws');
    replacement.on('error', () => {});
    replacement.on('open', () => replacement.send(JSON.stringify({ type: 'auth', token: `Bearer ${token}` })));
    await waitUntil(() => connections >= 2 && manager.isConnected());
    const after = await peer.sendMessage(room.id, 'after reconnect', accepted.id);
    await waitUntil(() => events.some((event) => event.message_id === after.id));
    assert.equal((await client.getRoom(room.id)).messages.at(-1).id, after.id);
    assert.equal(after.reply_to, accepted.id);
    client.logout();
    assert(!manager.isConnected());
    console.log(JSON.stringify({ suite: 'typescript-sdk', passed: true, scenarios: 12 }));
  } finally {
    replacement?.close();
    client.disconnect();
    peer.disconnect();
  }
})().catch((error) => {
  // Axios errors contain credentials in config; only publish the safe message.
  console.error(`TypeScript smoke failed: ${error.message}`);
  process.exitCode = 1;
});
