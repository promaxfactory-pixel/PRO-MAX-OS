import { useState, useEffect, useRef, useCallback } from "react";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import {
  Send, Bot, User, Settings, MessageSquare, Plus, Trash2,
  Eye, EyeOff, Loader2, Zap, TrendingUp, Warehouse,
  DollarSign, BarChart3, Lightbulb, X, ChevronDown, Sparkles,
  TestTube, Save, History, PanelRightOpen, CheckCircle2, AlertCircle
} from "lucide-react";

type ContextType = "general" | "financial" | "production" | "inventory" | "hr";

interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: Date;
}

interface ChatSession {
  id: string;
  title: string;
  contextType: ContextType;
  messages: ChatMessage[];
  created_at: Date;
}

interface AiSettings {
  api_key: string;
  model: string;
  max_tokens: number;
  temperature: number;
}

interface Suggestion {
  id: string;
  label: string;
  prompt: string;
}

const CONTEXTS: { value: ContextType; label: string }[] = [
  { value: "general", label: "عام" },
  { value: "financial", label: "مالي" },
  { value: "production", label: "إنتاج" },
  { value: "inventory", label: "مخزون" },
  { value: "hr", label: "موارد بشرية" },
];

const QUICK_ACTIONS: Suggestion[] = [
  { id: "sales", label: "تحليل المبيعات", prompt: "حلل بيانات المبيعات الحالية وقدم تقريراً مفصلاً" },
  { id: "inventory", label: "مراجعة المخزون", prompt: "راجع حالة المخزون وحدد المنتجات التي تحتاج إعادة طلب" },
  { id: "profit", label: "تحليل الأرباح", prompt: "حلل الأرباح والخسائر للفترة الحالية" },
  { id: "performance", label: "تقييم الأداء", prompt: "قدم تقييماً شاملاً لأداء الشركة" },
  { id: "improve", label: "اقتراحات التحسين", prompt: "اقترح تحسينات لتطوير العمليات التشغيلية" },
];

const DEFAULT_SETTINGS: AiSettings = {
  api_key: "",
  model: "gpt-4o",
  max_tokens: 2048,
  temperature: 0.7,
};

function generateId() {
  return Math.random().toString(36).substring(2, 11);
}

function TypingDots() {
  return (
    <div className="flex items-center gap-1.5 px-1">
      <div className="w-2 h-2 rounded-full bg-surface-500 animate-bounce" style={{ animationDelay: "0ms" }} />
      <div className="w-2 h-2 rounded-full bg-surface-500 animate-bounce" style={{ animationDelay: "150ms" }} />
      <div className="w-2 h-2 rounded-full bg-surface-500 animate-bounce" style={{ animationDelay: "300ms" }} />
    </div>
  );
}

function escapeHtml(str: string): string {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function renderMessageContent(content: string) {
  const lines = content.split("\n");
  return lines.map((line, i) => {
    if (line.startsWith("```")) {
      const code = lines.slice(i + 1, lines.findIndex((l, j) => j > i && l.startsWith("```"))).join("\n");
      if (code) {
        return (
          <pre key={i} className="bg-surface-900 rounded-lg p-3 my-2 text-xs font-mono text-surface-300 overflow-x-auto" dir="ltr">
            {code}
          </pre>
        );
      }
    }
    if (/^#{1,3}\s/.test(line)) {
      const level = line.match(/^#+/)?.[0].length || 1;
      const text = line.replace(/^#+\s*/, "");
      const Tag = (`h${Math.min(level + 2, 5)}`) as keyof JSX.IntrinsicElements;
      return <Tag key={i} className="font-bold text-white mt-3 mb-1 text-sm">{text}</Tag>;
    }
    if (/^\*\s/.test(line)) {
      return <li key={i} className="text-sm text-surface-300 mr-4 list-disc">{line.replace(/^\*\s*/, "")}</li>;
    }
    if (/^\d+\.\s/.test(line)) {
      return <li key={i} className="text-sm text-surface-300 mr-4 list-decimal">{line.replace(/^\d+\.\s*/, "")}</li>;
    }
    if (line.trim() === "") return <br key={i} />;
    const escaped = escapeHtml(line);
    const formatted = escaped
      .replace(/\*\*(.*?)\*\*/g, '<strong class="text-white">$1</strong>')
      .replace(/\*(.*?)\*/g, '<em>$1</em>');
    return <p key={i} className="text-sm text-surface-300" dangerouslySetInnerHTML={{ __html: formatted }} />;
  });
}

export default function AiAssistantPage() {
  const navigate = useNavigate();

  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [contextType, setContextType] = useState<ContextType>("general");
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isTyping, setIsTyping] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [settings, setSettings] = useState<AiSettings>(DEFAULT_SETTINGS);
  const [settingsLoaded, setSettingsLoaded] = useState(false);
  const [testing, setTesting] = useState(false);
  const [showApiKey, setShowApiKey] = useState(false);
  const [showHistory, setShowHistory] = useState(true);
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const activeSession = sessions.find((s) => s.id === activeSessionId);

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [activeSession?.messages, isTyping, scrollToBottom]);

  useEffect(() => {
    invoke<Partial<AiSettings>>("get_ai_settings")
      .then((s) => {
        setSettings((prev) => ({ ...prev, ...s }));
        setSettingsLoaded(true);
      })
      .catch(() => setSettingsLoaded(true));
  }, []);

  useEffect(() => {
    const loadSuggestions = async () => {
      if (!contextType) return;
      try {
        const data = await invoke<Suggestion[]>("ai_suggest_actions", { contextType });
        setSuggestions(data);
      } catch {
        setSuggestions([]);
      }
    };
    loadSuggestions();
  }, [contextType]);

  const createNewSession = useCallback(() => {
    const id = generateId();
    const ctxLabel = CONTEXTS.find((c) => c.value === contextType)?.label || "عام";
    const session: ChatSession = {
      id,
      title: `محادثة ${ctxLabel}`,
      contextType,
      messages: [],
      created_at: new Date(),
    };
    setSessions((prev) => [session, ...prev]);
    setActiveSessionId(id);
    setTimeout(() => inputRef.current?.focus(), 100);
  }, [contextType]);

  useEffect(() => {
    if (!activeSessionId && sessions.length === 0 && settingsLoaded) {
      createNewSession();
    }
  }, [settingsLoaded, activeSessionId, sessions.length, createNewSession]);

  const handleSend = async () => {
    const text = input.trim();
    if (!text || isLoading) return;
    setInput("");
    setError(null);

    const userMsg: ChatMessage = {
      id: generateId(),
      role: "user",
      content: text,
      timestamp: new Date(),
    };

    if (!activeSessionId) {
      const ctxLabel = CONTEXTS.find((c) => c.value === contextType)?.label || "عام";
      const sessionId = generateId();
      setSessions([{
        id: sessionId,
        title: text.substring(0, 40) + (text.length > 40 ? "..." : ""),
        contextType,
        messages: [userMsg],
        created_at: new Date(),
      }]);
      setActiveSessionId(sessionId);
    } else {
      setSessions((prev) =>
        prev.map((s) =>
          s.id === activeSessionId
            ? { ...s, messages: [...s.messages, userMsg], title: s.messages.length === 0 ? text.substring(0, 40) : s.title }
            : s
        )
      );
    }

    setIsLoading(true);
    setIsTyping(true);

    try {
      const result: { reply: string; model: string; provider: string; token_estimate: number } =
        await invoke("chat_with_ai", {
          input: { message: text, context_type: contextType, provider: "openai" },
        });

      const assistantMsg: ChatMessage = {
        id: generateId(),
        role: "assistant",
        content: result.reply,
        timestamp: new Date(),
      };

      setSessions((prev) =>
        prev.map((s) => s.id === activeSessionId ? { ...s, messages: [...s.messages, assistantMsg] } : s)
      );
    } catch (err: unknown) {
      const errMsg: ChatMessage = {
        id: generateId(),
        role: "system",
        content: err instanceof Error ? err.message : String(err) || "حدث خطأ غير متوقع",
        timestamp: new Date(),
      };
      setSessions((prev) =>
        prev.map((s) => s.id === activeSessionId ? { ...s, messages: [...s.messages, errMsg] } : s)
      );
    } finally {
      setIsTyping(false);
      setIsLoading(false);
    }
  };

  const handleQuickAction = async (action: Suggestion) => {
    setInput(action.prompt);
    setTimeout(() => handleSend(), 100);
  };

  const handleSaveSettings = async () => {
    try {
      await invoke("save_ai_provider_settings", {
        provider: "openai",
        apiKey: settings.api_key,
        model: settings.model,
      });
      setStatusMessage({ type: "success", text: "تم حفظ الإعدادات بنجاح" });
      setTimeout(() => setStatusMessage(null), 3000);
    } catch (err: unknown) {
      setStatusMessage({ type: "error", text: err instanceof Error ? err.message : String(err) || "فشل حفظ الإعدادات" });
    }
  };

  const handleTestConnection = async () => {
    setTesting(true);
    setError(null);
    try {
      const result: { configured: boolean; provider: string; model: string; message: string } =
        await invoke("test_ai_connection", { provider: "openai" });
      setStatusMessage({
        type: result.configured ? "success" : "error",
        text: result.message,
      });
      setTimeout(() => setStatusMessage(null), 5000);
    } catch (err: unknown) {
      setStatusMessage({ type: "error", text: err instanceof Error ? err.message : String(err) || "فشل الاتصال" });
    } finally {
      setTesting(false);
    }
  };

  const deleteSession = (id: string) => {
    setSessions((prev) => prev.filter((s) => s.id !== id));
    if (activeSessionId === id) {
      const remaining = sessions.filter((s) => s.id !== id);
      setActiveSessionId(remaining[0]?.id || null);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const currentMessages = activeSession?.messages || [];

  return (
    <div className="h-[calc(100vh-8rem)] flex gap-6">
      {showHistory && (
        <div className="w-72 flex-shrink-0 space-y-4 overflow-y-auto">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-bold text-white flex items-center gap-2">
              <MessageSquare className="w-4 h-4 text-gold-400" />
              المحادثات
            </h3>
            <Button variant="ghost" size="sm" onClick={() => setShowSettings(true)}>
              <Settings className="w-4 h-4 text-surface-400" />
            </Button>
          </div>

          <select
            value={contextType}
            onChange={(e) => setContextType(e.target.value as ContextType)}
            className="input-field w-full text-sm"
          >
            {CONTEXTS.map((c) => <option key={c.value} value={c.value}>{c.label}</option>)}
          </select>

          <Button
            variant="outline"
            className="w-full"
            size="sm"
            icon={<Plus className="w-4 h-4" />}
            onClick={() => {
              const id = generateId();
              const ctxLabel = CONTEXTS.find((c) => c.value === contextType)?.label || "عام";
              setSessions((prev) => [{
                id,
                title: `محادثة ${ctxLabel}`,
                contextType,
                messages: [],
                created_at: new Date(),
              }, ...prev]);
              setActiveSessionId(id);
            }}
          >
            محادثة جديدة
          </Button>

          <div className="space-y-1">
            {sessions.map((s) => (
              <div
                key={s.id}
                onClick={() => setActiveSessionId(s.id)}
                className={cn(
                  "flex items-center justify-between p-2 rounded-lg cursor-pointer transition-all text-sm",
                  activeSessionId === s.id
                    ? "bg-brand-800/30 border border-brand-500/40 text-white"
                    : "text-surface-400 hover:bg-surface-800/50 hover:text-surface-300"
                )}
              >
                <span className="truncate flex-1">{s.title}</span>
                <Trash2
                  className="w-3.5 h-3.5 text-surface-600 hover:text-red-400 flex-shrink-0"
                  onClick={(e) => { e.stopPropagation(); deleteSession(s.id); }}
                />
              </div>
            ))}
            {sessions.length === 0 && (
              <p className="text-xs text-surface-500 text-center py-4">لا توجد محادثات</p>
            )}
          </div>

          {suggestions.length > 0 && (
            <div className="pt-2">
              <h4 className="text-xs text-surface-500 mb-2 flex items-center gap-1">
                <Sparkles className="w-3 h-3" />
                الإجراءات المقترحة
              </h4>
              {suggestions.map((s) => (
                <button
                  key={s.id}
                  onClick={() => handleQuickAction(s)}
                  className="w-full text-right text-xs text-surface-400 hover:text-white p-1.5 rounded-lg hover:bg-surface-800/50 transition-all"
                >
                  {s.label}
                </button>
              ))}
            </div>
          )}

          <button
            onClick={() => setShowHistory(false)}
            className="w-full text-xs text-surface-500 hover:text-surface-300 py-2"
          >
            إخفاء الشريط الجانبي
          </button>
        </div>
      )}

      {!showHistory && (
        <button
          onClick={() => setShowHistory(true)}
          className="flex-shrink-0 self-start p-2 mt-2 text-surface-500 hover:text-white rounded-lg hover:bg-surface-800/50"
        >
          <PanelRightOpen className="w-5 h-5" />
        </button>
      )}

      <div className="flex-1 flex flex-col bg-surface-900/50 rounded-2xl border border-surface-700/50 overflow-hidden">
        {currentMessages.length === 0 && !isTyping && (
          <div className="flex-1 flex flex-col items-center justify-center p-8 text-center">
            <Bot className="w-16 h-16 text-gold-400/30 mb-4" />
            <h2 className="text-xl font-bold text-white mb-2">المساعد الذكي</h2>
            <p className="text-surface-400 text-sm mb-6">
              اسأل عن أي شيء يتعلق بنظام إدارة الشركة
            </p>
            <div className="flex flex-wrap gap-2 justify-center max-w-lg">
              {QUICK_ACTIONS.map((action) => (
                <button
                  key={action.id}
                  onClick={() => handleQuickAction(action)}
                  className="px-4 py-2 bg-surface-800/50 border border-surface-700 rounded-xl text-sm text-surface-300 hover:border-brand-500/40 hover:text-white transition-all"
                >
                  {action.label}
                </button>
              ))}
            </div>
            <div className="flex flex-wrap gap-2 justify-center mt-3">
              {CONTEXTS.filter((c) => c.value !== contextType).map((c) => (
                <button
                  key={c.value}
                  onClick={() => setContextType(c.value)}
                  className="px-3 py-1 bg-surface-800/30 rounded-lg text-xs text-surface-500 hover:text-surface-300"
                >
                  {c.label}
                </button>
              ))}
            </div>
          </div>
        )}

        {currentMessages.length > 0 && (
          <div className="flex-1 overflow-y-auto p-4 space-y-4">
            {currentMessages.map((msg) => {
              if (msg.role === "system") {
                return (
                  <div key={msg.id} className="text-center">
                    <span className="inline-block text-xs text-surface-500 bg-surface-800/50 px-3 py-1 rounded-full">
                      {msg.content}
                    </span>
                  </div>
                );
              }
              const isUser = msg.role === "user";
              return (
                <div key={msg.id} className={cn("flex", isUser ? "justify-start" : "justify-end")}>
                  <div className="flex items-start gap-3 max-w-[80%]">
                    {!isUser && (
                      <div className="w-8 h-8 rounded-full bg-gold-500/20 flex items-center justify-center flex-shrink-0 mt-1">
                        <Bot className="w-4 h-4 text-gold-400" />
                      </div>
                    )}
                    <div
                      className={cn(
                        "p-4",
                        isUser
                          ? "bg-brand-700/30 rounded-2xl rounded-tr-md"
                          : "bg-surface-800/80 border border-surface-700 rounded-2xl rounded-tl-md"
                      )}
                    >
                      {renderMessageContent(msg.content)}
                    </div>
                    {isUser && (
                      <div className="w-8 h-8 rounded-full bg-brand-800/30 flex items-center justify-center flex-shrink-0 mt-1">
                        <User className="w-4 h-4 text-brand-400" />
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
            {isTyping && (
              <div className="flex justify-end">
                <div className="flex items-start gap-3 max-w-[80%]">
                  <div className="w-8 h-8 rounded-full bg-gold-500/20 flex items-center justify-center flex-shrink-0 mt-1">
                    <Bot className="w-4 h-4 text-gold-400" />
                  </div>
                  <div className="p-4 bg-surface-800/80 border border-surface-700 rounded-2xl rounded-tl-md">
                    <TypingDots />
                  </div>
                </div>
              </div>
            )}
            <div ref={messagesEndRef} />
          </div>
        )}

        <div className="p-4 border-t border-surface-700/50 bg-surface-900/80">
          {error && (
            <div className="flex items-center gap-2 mb-3 text-sm text-red-400">
              <AlertCircle className="w-4 h-4" />
              {error}
            </div>
          )}
          <div className="flex items-center gap-3">
            <input
              ref={inputRef}
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="اكتب رسالتك هنا..."
              className="input-field flex-1"
              disabled={isLoading}
              dir="rtl"
            />
            <Button
              onClick={handleSend}
              disabled={!input.trim() || isLoading}
              icon={isLoading ? <Loader2 className="w-4 h-4 animate-spin" /> : <Send className="w-4 h-4" />}
            >
              إرسال
            </Button>
          </div>
        </div>
      </div>

      {showSettings && (
        <div className="fixed inset-0 z-50 bg-black/60 flex items-center justify-center" onClick={() => setShowSettings(false)}>
          <div
            className="bg-surface-900 rounded-2xl border border-surface-700 w-full max-w-lg max-h-[90vh] overflow-y-auto p-6"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-lg font-bold text-white flex items-center gap-2">
                <Settings className="w-5 h-5 text-gold-400" />
                إعدادات الذكاء الاصطناعي
              </h3>
              <button onClick={() => setShowSettings(false)} className="text-surface-500 hover:text-white">
                <X className="w-5 h-5" />
              </button>
            </div>

            {statusMessage && (
              <div className={cn(
                "flex items-center gap-2 p-3 rounded-xl text-sm mb-4",
                statusMessage.type === "success"
                  ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/30"
                  : "bg-red-500/10 text-red-400 border border-red-500/30"
              )}>
                {statusMessage.type === "success" ? <CheckCircle2 className="w-4 h-4" /> : <AlertCircle className="w-4 h-4" />}
                {statusMessage.text}
              </div>
            )}

            <div className="space-y-4">
              <div className="input-group">
                <label className="input-label">مفتاح API</label>
                <div className="relative">
                  <input
                    type={showApiKey ? "text" : "password"}
                    value={settings.api_key}
                    onChange={(e) => setSettings((prev) => ({ ...prev, api_key: e.target.value }))}
                    className="input-field w-full ltr text-left"
                    dir="ltr"
                    placeholder="sk-..."
                  />
                  <button
                    onClick={() => setShowApiKey(!showApiKey)}
                    className="absolute left-2 top-1/2 -translate-y-1/2 text-surface-500 hover:text-surface-300"
                  >
                    {showApiKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  </button>
                </div>
              </div>

              <div className="input-group">
                <label className="input-label">النموذج</label>
                <select
                  value={settings.model}
                  onChange={(e) => setSettings((prev) => ({ ...prev, model: e.target.value }))}
                  className="input-field w-full"
                >
                  <option value="gpt-4o">GPT-4o</option>
                  <option value="gpt-4o-mini">GPT-4o Mini</option>
                  <option value="gpt-3.5-turbo">GPT-3.5 Turbo</option>
                </select>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="input-group">
                  <label className="input-label">الحد الأقصى للرموز</label>
                  <input
                    type="number"
                    value={settings.max_tokens}
                    onChange={(e) => setSettings((prev) => ({ ...prev, max_tokens: Number(e.target.value) }))}
                    className="input-field w-full"
                    min={256}
                    max={8192}
                  />
                </div>
                <div className="input-group">
                  <label className="input-label">الحرارة (Temperature)</label>
                  <input
                    type="range"
                    min="0"
                    max="2"
                    step="0.1"
                    value={settings.temperature}
                    onChange={(e) => setSettings((prev) => ({ ...prev, temperature: Number(e.target.value) }))}
                    className="w-full"
                  />
                  <span className="text-xs text-surface-500">{settings.temperature.toFixed(1)}</span>
                </div>
              </div>

              <div className="flex items-center gap-3 pt-2">
                <Button onClick={handleSaveSettings} icon={<Save className="w-4 h-4" />}>
                  حفظ
                </Button>
                <Button
                  variant="outline"
                  onClick={handleTestConnection}
                  loading={testing}
                  icon={<TestTube className="w-4 h-4" />}
                >
                  اختبار الاتصال
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
