import { useEffect, useRef } from 'react';

interface Message {
  id: string;
  content: string;
  sender: { displayName?: string; type?: string };
  createdAt: string;
}

interface Props {
  messages: Message[];
  onSummarize?: (messageId: string) => void;
}

function isAI(sender: { type?: string }): boolean {
  return sender.type === 'ai';
}

function formatTime(ts: string): string {
  return new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

export default function MessageList({ messages, onSummarize }: Props) {
  const bottom = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottom.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages.length]);

  return (
    <div className="flex-1 overflow-y-auto p-4 space-y-4">
      {messages.map((msg) => (
        <div key={msg.id} className={`flex gap-3 ${isAI(msg.sender) ? 'bg-brand-50 rounded-lg p-3' : ''}`}>
          <div className="w-8 h-8 rounded-full flex items-center justify-center text-white text-sm font-bold shrink-0 bg-brand-600">
            {isAI(msg.sender) ? '🤖' : msg.sender.displayName?.[0]?.toUpperCase()}
          </div>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span className="font-semibold text-sm text-gray-900">
                {msg.sender.displayName}
                {isAI(msg.sender) && (
                  <span className="ml-1 text-xs bg-brand-100 text-brand-700 px-1.5 py-0.5 rounded">AI</span>
                )}
              </span>
              <span className="text-xs text-gray-400">{formatTime(msg.createdAt)}</span>
            </div>
            <div className="text-gray-800 text-sm whitespace-pre-wrap break-words">{msg.content}</div>
            {isAI(msg.sender) && onSummarize && (
              <button onClick={() => onSummarize(msg.id)} className="mt-1 text-xs text-brand-600 hover:text-brand-700 hover:underline">
                ✨ Summarize
              </button>
            )}
          </div>
        </div>
      ))}
      <div ref={bottom} />
    </div>
  );
}
