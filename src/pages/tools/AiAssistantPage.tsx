import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import Button from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import {
  Send, Bot, User, Settings, MessageSquare, Plus, Trash2,
  Eye, EyeOff, Loader2,
  X, Sparkles,
  TestTube, Save, PanelRightOpen, CheckCircle2, AlertCircle
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

interface ProviderStatus {
  id: string;
  label: string;
  model: string;
  configured: boolean;
  enabled: boolean;
  requires_key: boolean;
  free_tier: boolean;
  message: string;
}

interface ProviderSettings {
  provider: string;
  label: string;
  model: string;
  base_url: string;
  enabled: boolean;
  has_key: boolean;
  requires_key: boolean;
  models: string[];
}

interface Suggestion {
  id: string;
  label: string;
  prompt: string;
}

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
  const { t } = useTranslation();

  const CONTEXTS: { value: ContextType; label: string }[] = [
    { value: "general", label: t("tools.aiAssistant.contextGeneral") },
    { value: "financial", label: t("tools.aiAssistant.contextFinancial") },
    { value: "production", label: t("tools.aiAssistant.contextProduction") },
    { value: "inventory", label: t("tools.aiAssistant.contextInventory") },
    { value: "hr", label: t("tools.aiAssistant.contextHr") },
  ];

  const QUICK_ACTIONS: Suggestion[] = [
    { id: "sales", label: t("tools.aiAssistant.actionSalesLabel"), prompt: t("tools.aiAssistant.actionSalesPrompt") },
    { id: "inventory", label: t("tools.aiAssistant.actionInventoryLabel"), prompt: t("tools.aiAssistant.actionInventoryPrompt") },
    { id: "profit", label: t("tools.aiAssistant.actionProfitLabel"), prompt: t("tools.aiAssistant.actionProfitPrompt") },
    { id: "performance", label: t("tools.aiAssistant.actionPerformanceLabel"), prompt: t("tools.aiAssistant.actionPerformancePrompt") },
    { id: "improve", label: t("tools.aiAssistant.actionImproveLabel"), prompt: t("tools.aiAssistant.actionImprovePrompt") },
  ];

  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [contextType, setContextType] = useState<ContextType>("general");
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isTyping, setIsTyping] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showApiKey, setShowApiKey] = useState(false);
  const [showHistory, setShowHistory] = useState(true);
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  const [providers, setProviders] = useState<ProviderStatus[]>([]);
  const [chatProvider, setChatProvider] = useState("auto");
  const [activeProviderTab, setActiveProviderTab] = useState<string | null>(null);
  const [providerSettings, setProviderSettings] = useState<ProviderSettings | null>(null);
  const [providerApiKey, setProviderApiKey] = useState("");
  const [providerModel, setProviderModel] = useState("");
  const [providerBaseUrl, setProviderBaseUrl] = useState("");
  const [providerEnabled, setProviderEnabled] = useState(true);
  const [providerTesting, setProviderTesting] = useState(false);
  const [lastUsedProvider, setLastUsedProvider] = useState<string | null>(null);

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
    invoke<ProviderStatus[]>("ai_provider_statuses")
      .then(setProviders)
      .catch(() => setProviders([]));
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
    const ctxLabel = CONTEXTS.find((c) => c.value === contextType)?.label || t("tools.aiAssistant.contextGeneral");
    const session: ChatSession = {
      id,
      title: t("tools.aiAssistant.sessionTitle", { context: ctxLabel }),
      contextType,
      messages: [],
      created_at: new Date(),
    };
    setSessions((prev) => [session, ...prev]);
    setActiveSessionId(id);
    setTimeout(() => inputRef.current?.focus(), 100);
  }, [contextType, t]);

  useEffect(() => {
    if (!activeSessionId && sessions.length === 0) {
      createNewSession();
    }
  }, [activeSessionId, sessions.length, createNewSession]);

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
      const result: { reply: string; model: string; provider: string; provider_label: string; used_fallback: boolean; attempts: string[] } =
        await invoke("ai_chat_with_provider", {
          message: text,
          context_type: contextType,
          provider: chatProvider === "auto" ? null : chatProvider,
        });

      const assistantMsg: ChatMessage = {
        id: generateId(),
        role: "assistant",
        content: result.reply,
        timestamp: new Date(),
      };

      setLastUsedProvider(`${result.provider_label} · ${result.model}${result.used_fallback ? ` (${t("tools.aiAssistant.fallbackUsed")})` : ""}`);

      setSessions((prev) =>
        prev.map((s) => s.id === activeSessionId ? { ...s, messages: [...s.messages, assistantMsg] } : s)
      );
    } catch (err: unknown) {
      const errMsg: ChatMessage = {
        id: generateId(),
        role: "system",
        content: err instanceof Error ? err.message : String(err) || t("tools.aiAssistant.unexpectedError"),
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

  const loadProviderSettings = useCallback(async (id: string) => {
    try {
      const s = await invoke<ProviderSettings>("ai_get_provider_settings", { provider: id });
      setProviderSettings(s);
      setProviderModel(s.model);
      setProviderBaseUrl(s.base_url);
      setProviderEnabled(s.enabled);
      setProviderApiKey("");
      setShowApiKey(false);
    } catch (err: unknown) {
      setStatusMessage({ type: "error", text: err instanceof Error ? err.message : String(err) });
    }
  }, []);

  const openSettings = useCallback(async () => {
    setShowSettings(true);
    setStatusMessage(null);
    const first = providers.find((p) => p.configured)?.id || providers[0]?.id || "openai";
    setActiveProviderTab(first);
    await loadProviderSettings(first);
  }, [providers, loadProviderSettings]);

  const handleSaveProviderSettings = async () => {
    if (!providerSettings) return;
    try {
      await invoke("ai_save_provider_config", {
        provider: providerSettings.provider,
        apiKey: providerApiKey || null,
        model: providerModel,
        baseUrl: providerBaseUrl,
        enabled: providerEnabled,
      });
      const statuses = await invoke<ProviderStatus[]>("ai_provider_statuses");
      setProviders(statuses);
      await loadProviderSettings(providerSettings.provider);
      setStatusMessage({ type: "success", text: t("tools.aiAssistant.settingsSaved") });
      setTimeout(() => setStatusMessage(null), 3000);
    } catch (err: unknown) {
      setStatusMessage({ type: "error", text: err instanceof Error ? err.message : String(err) || t("tools.aiAssistant.settingsSaveFailed") });
    }
  };

  const handleTestProvider = async () => {
    if (!providerSettings) return;
    setProviderTesting(true);
    setError(null);
    try {
      const result = await invoke<ProviderStatus>("ai_test_provider", { provider: providerSettings.provider });
      setStatusMessage({
        type: result.configured && (result.message.startsWith("Connection successful") || result.message.startsWith("OK")) ? "success" : "error",
        text: result.message,
      });
      setTimeout(() => setStatusMessage(null), 5000);
    } catch (err: unknown) {
      setStatusMessage({ type: "error", text: err instanceof Error ? err.message : String(err) || t("tools.connectionFailed") });
    } finally {
      setProviderTesting(false);
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
              {t("tools.aiAssistant.conversations")}
            </h3>
            <Button variant="ghost" size="sm" onClick={openSettings}>
              <Settings className="w-4 h-4 text-surface-400" />
            </Button>
          </div>

          <select
            value={contextType}
            onChange={(e) => setContextType(e.target.value as ContextType)}
            className="input-field w-full text-sm"
            aria-label={t("tools.aiAssistant.contextAria")}
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
              const ctxLabel = CONTEXTS.find((c) => c.value === contextType)?.label || t("tools.aiAssistant.contextGeneral");
              setSessions((prev) => [{
                id,
                title: t("tools.aiAssistant.sessionTitle", { context: ctxLabel }),
                contextType,
                messages: [],
                created_at: new Date(),
              }, ...prev]);
              setActiveSessionId(id);
            }}
          >
            {t("tools.aiAssistant.newConversation")}
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
              <p className="text-xs text-surface-500 text-center py-4">{t("tools.aiAssistant.noConversations")}</p>
            )}
          </div>

          {suggestions.length > 0 && (
            <div className="pt-2">
              <h4 className="text-xs text-surface-500 mb-2 flex items-center gap-1">
                <Sparkles className="w-3 h-3" />
                {t("tools.suggestedActions")}
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
            {t("tools.aiAssistant.hideSidebar")}
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
            <h2 className="text-xl font-bold text-white mb-2">{t("tools.aiAssistant.title")}</h2>
            <p className="text-surface-400 text-sm mb-6">
              {t("tools.aiAssistant.subtitle")}
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
            <select
              value={chatProvider}
              onChange={(e) => setChatProvider(e.target.value)}
              className="input-field w-44 flex-shrink-0 text-xs"
              aria-label={t("tools.aiAssistant.chatProviderAria")}
              dir="rtl"
            >
              <option value="auto">{t("tools.aiAssistant.autoProvider")}</option>
              {providers.map((p) => (
                <option key={p.id} value={p.id} disabled={!p.configured}>
                  {p.label}{p.configured ? "" : ` (${t("tools.aiAssistant.notConfigured")})`}
                </option>
              ))}
            </select>
            <input
              ref={inputRef}
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={t("tools.aiAssistant.inputPlaceholder")}
              className="input-field flex-1"
              disabled={isLoading}
              dir="rtl"
              aria-label={t("tools.aiAssistant.inputAria")}
            />
            <Button
              onClick={handleSend}
              disabled={!input.trim() || isLoading}
              icon={isLoading ? <Loader2 className="w-4 h-4 animate-spin" /> : <Send className="w-4 h-4" />}
            >
              {t("tools.aiAssistant.send")}
            </Button>
          </div>
          {lastUsedProvider && (
            <p className="mt-2 text-[11px] text-surface-500 flex items-center gap-1.5">
              <Sparkles className="w-3 h-3 text-gold-400/70" />
              {t("tools.aiAssistant.respondedBy", { provider: lastUsedProvider })}
            </p>
          )}
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
                {t("tools.aiAssistant.settingsTitle")}
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

            {/* Provider tabs */}
            <div className="flex flex-wrap gap-1.5 mb-5">
              {providers.map((p) => (
                <button
                  key={p.id}
                  onClick={() => { setActiveProviderTab(p.id); loadProviderSettings(p.id); }}
                  className={cn(
                    "inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs border transition-all",
                    activeProviderTab === p.id
                      ? "bg-brand-800/40 border-brand-500/50 text-white"
                      : "border-surface-700 text-surface-400 hover:text-surface-200 hover:border-surface-500"
                  )}
                >
                  {p.configured
                    ? <CheckCircle2 className="w-3 h-3 text-emerald-400" />
                    : <AlertCircle className="w-3 h-3 text-surface-600" />}
                  <span className="truncate max-w-[110px]">{p.label}</span>
                  {p.free_tier && <span className="opacity-60">{t("tools.aiFileImport.freeTier")}</span>}
                </button>
              ))}
            </div>

            {providerSettings && (
              <div className="space-y-4">
                <div className="flex items-center justify-between text-xs text-surface-400">
                  <span className="flex items-center gap-1.5">
                    {providerSettings.requires_key
                      ? t("tools.aiAssistant.requiresKey")
                      : t("tools.aiAssistant.noKeyRequired")}
                  </span>
                  <span className={cn(
                    "px-2 py-0.5 rounded-md",
                    providerSettings.has_key || !providerSettings.requires_key
                      ? "bg-emerald-500/10 text-emerald-400"
                      : "bg-amber-500/10 text-amber-400"
                  )}>
                    {providerSettings.has_key || !providerSettings.requires_key
                      ? t("tools.aiAssistant.hasKey")
                      : t("tools.aiAssistant.notConfigured")}
                  </span>
                </div>

                {providerSettings.requires_key && (
                  <div className="input-group">
                    <label className="input-label">{t("tools.aiAssistant.apiKeyLabel")}</label>
                    <div className="relative">
                      <input
                        type={showApiKey ? "text" : "password"}
                        value={providerApiKey}
                        onChange={(e) => setProviderApiKey(e.target.value)}
                        className="input-field w-full ltr text-left"
                        dir="ltr"
                        placeholder={providerSettings.has_key ? "••••••••••••••••" : "sk-..."}
                        aria-label={t("tools.aiAssistant.apiKeyAria")}
                      />
                      <button
                        onClick={() => setShowApiKey(!showApiKey)}
                        className="absolute left-2 top-1/2 -translate-y-1/2 text-surface-500 hover:text-surface-300"
                      >
                        {showApiKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                      </button>
                    </div>
                    {providerSettings.has_key && (
                      <p className="text-[11px] text-surface-500 mt-1">{t("tools.aiAssistant.enterNewKey")}</p>
                    )}
                  </div>
                )}

                <div className="input-group">
                  <label className="input-label">{t("tools.aiAssistant.modelLabel")}</label>
                  <select
                    value={providerModel}
                    onChange={(e) => setProviderModel(e.target.value)}
                    className="input-field w-full"
                    aria-label={t("tools.aiAssistant.modelAria")}
                  >
                    {providerSettings.models.map((m) => (
                      <option key={m} value={m}>{m}</option>
                    ))}
                  </select>
                </div>

                <div className="input-group">
                  <label className="input-label">{t("tools.aiAssistant.baseUrlLabel")}</label>
                  <input
                    type="text"
                    value={providerBaseUrl}
                    onChange={(e) => setProviderBaseUrl(e.target.value)}
                    className="input-field w-full ltr text-left"
                    dir="ltr"
                    aria-label={t("tools.aiAssistant.baseUrlAria")}
                  />
                </div>

                <label className="flex items-center gap-2 text-sm text-surface-300 cursor-pointer select-none">
                  <input
                    type="checkbox"
                    checked={providerEnabled}
                    onChange={(e) => setProviderEnabled(e.target.checked)}
                    className="accent-emerald-500 w-4 h-4"
                  />
                  {t("tools.aiAssistant.enabledLabel")}
                </label>

                <div className="flex items-center gap-3 pt-2">
                  <Button onClick={handleSaveProviderSettings} icon={<Save className="w-4 h-4" />}>
                    {t("common.save")}
                  </Button>
                  <Button
                    variant="outline"
                    onClick={handleTestProvider}
                    loading={providerTesting}
                    icon={<TestTube className="w-4 h-4" />}
                  >
                    {t("tools.aiAssistant.testConnection")}
                  </Button>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
