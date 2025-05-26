"use client";

import type React from "react";
import Image from "next/image";
import { useState, useRef, useEffect } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import {
  PlusCircle,
  X,
  ArrowUp,
  Menu,
  ChevronRight,
  ChevronLeft,
  Tag,
  Plus,
} from "lucide-react";
import ResizablePanel from "./resizable-panel";
import { CodeRegion } from "@/app/editor/page";
import { useKeyboardShortcut } from "@/hooks/use-keyboard-shortcut";
import { useChat, type Message as AiMessage } from "ai/react";
import type { JSONValue } from "ai";
import { cn } from "@/lib/utils";

type CodeChangeData = {
  type: "suggest_code_change";
  startLine: number;
  endLine: number;
  suggestedCode: string;
  explanation: string;
};

type CommentChange = {
  type: "comment" | "rename";
  startLine: number;
  endLine: number;
  oldText: string;
  newText: string;
  explanation: string;
};

type CommentChangeData = {
  type: "suggest_comment_change";
  changes: CommentChange[];
};

type ToolCall = {
  type: "suggest_code_change" | "suggest_comment_change";
  data: CodeChangeData | CommentChangeData;
};

interface MessageData {
  codeRegions?: CodeRegion[];
  toolCalls?: ToolCall[];
}

interface Message extends Omit<AiMessage, "data"> {
  data?: MessageData;
}

interface ChatPanelProps {
  isOpen: boolean;
  onClose: () => void;
  selectedRegions: CodeRegion[];
  onRemoveRegion: (regionId: string) => void;
  onClearAllRegions: () => void;
  onMessageSent: () => void;
  onApplyChange?: (change: {
    fileName: string;
    startLine: number;
    endLine: number;
    suggestedCode: string;
  }) => void;
  onRejectChange?: (change: {
    fileName: string;
    startLine: number;
    endLine: number;
  }) => void;
}

export default function ChatPanel({
  isOpen,
  onClose,
  selectedRegions,
  onRemoveRegion,
  onClearAllRegions,
  onMessageSent,
  onApplyChange,
  onRejectChange,
}: ChatPanelProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [showTools, setShowTools] = useState(false);
  const [activeTagIndex, setActiveTagIndex] = useState(0);

  const {
    messages,
    isLoading,
    input,
    setInput,
    handleInputChange,
    append,
    setMessages,
  } = useChat({
    api: "/api/ask",
    id: "editor-chat",
    maxSteps: 5,
    onFinish: () => {
      onMessageSent();
    },
  });

  // Focus the textarea when the panel opens
  useEffect(() => {
    if (isOpen && textareaRef.current) {
      textareaRef.current.focus();
    }
  }, [isOpen]);

  // Reset active tag index when regions change
  useEffect(() => {
    if (
      selectedRegions.length > 0 &&
      activeTagIndex >= selectedRegions.length
    ) {
      setActiveTagIndex(Math.max(0, selectedRegions.length - 1));
    }
  }, [selectedRegions, activeTagIndex]);

  const handleSubmit = async () => {
    if (!input.trim() && !selectedRegions.length) return;

    const regions = selectedRegions.map((region) => ({
      id: region.id,
      start: region.start,
      end: region.end,
      code: region.code,
    }));
    // Save input to a variable
    const userInput = input;

    setInput("");
    onMessageSent();

    await append({
      content: userInput,
      role: "user",
      data: regions.length > 0 ? { codeRegions: regions } : undefined,
    });
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  // Add a function to toggle the tools dropdown
  const toggleTools = () => {
    setShowTools((prev) => !prev);
  };

  // Format a memory address from a line number
  const formatMemoryAddress = (lineNum: number): string => {
    return `0x${lineNum.toString(16).padStart(4, "0").toUpperCase()}`;
  };

  // Navigate to previous tag
  const prevTag = () => {
    setActiveTagIndex((prev) => (prev > 0 ? prev - 1 : prev));
  };

  // Navigate to next tag
  const nextTag = () => {
    setActiveTagIndex((prev) =>
      prev < selectedRegions.length - 1 ? prev + 1 : prev
    );
  };

  // Add keyboard shortcuts for tag navigation
  useKeyboardShortcut(
    "p",
    () => {
      if (activeTagIndex > 0) {
        prevTag();
      }
    },
    { metaKey: true }
  );

  useKeyboardShortcut(
    "n",
    () => {
      if (activeTagIndex < selectedRegions.length - 1) {
        nextTag();
      }
    },
    { metaKey: true }
  );

  // Render a tool call card
  const renderCommentChange = (
    toolCall: ToolCall & { data: CommentChangeData }
  ) => {
    const { changes } = toolCall.data;

    return (
      <div className="bg-[#1A1A1A] rounded-lg p-4 mb-4">
        <div className="flex items-center justify-between mb-2">
          <span className="text-sm text-white/70">
            Suggested{" "}
            {changes.length === 1 ? "Change" : `Changes (${changes.length})`}
          </span>
        </div>
        <div className="space-y-3">
          {changes.map((change: CommentChange, index: number) => (
            <div key={index} className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-xs text-white/50">
                  {change.type === "comment"
                    ? "Comment Change"
                    : "Rename Symbol"}
                </span>
                <div className="flex gap-2">
                  <button
                    className="px-2 py-1 text-xs rounded-md bg-[#1AC69C]/10 border border-[#1AC69C]/50 text-[#59F3A6]"
                    onClick={() =>
                      onApplyChange?.({
                        fileName: "",
                        startLine: change.startLine,
                        endLine: change.endLine,
                        suggestedCode: change.newText,
                      })
                    }
                  >
                    Apply
                  </button>
                  <button
                    className="px-2 py-1 text-xs rounded-md bg-[#FF32C6]/10 border border-[#FF32C6]/50 text-[#FF32C6]"
                    onClick={() =>
                      onRejectChange?.({
                        fileName: "",
                        startLine: change.startLine,
                        endLine: change.endLine,
                      })
                    }
                  >
                    Reject
                  </button>
                </div>
              </div>
              <div className="bg-[#0D0D0D] rounded p-2 text-sm space-y-1">
                <div className="text-white/50">From:</div>
                <pre className="text-white/90 overflow-x-auto">
                  <code>{change.oldText}</code>
                </pre>
                <div className="text-white/50 mt-2">To:</div>
                <pre className="text-white/90 overflow-x-auto">
                  <code>{change.newText}</code>
                </pre>
              </div>
              <p className="text-sm text-white/70">{change.explanation}</p>
            </div>
          ))}
        </div>
      </div>
    );
  };

  const renderToolCall = (toolCall: ToolCall | undefined) => {
    if (!toolCall) return null;

    if (toolCall.type === "suggest_comment_change") {
      return renderCommentChange(
        toolCall as ToolCall & { data: CommentChangeData }
      );
    }

    if (toolCall.type === "suggest_code_change") {
      const data = toolCall.data as CodeChangeData;
      const { startLine, endLine, suggestedCode, explanation } = data;

      return (
        <div className="bg-[#1A1A1A] rounded-lg p-4 mb-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm text-white/70">Suggested Code Change</span>
            <div className="flex gap-2">
              <button
                className="px-2 py-1 text-xs rounded-md bg-[#1AC69C]/10 border border-[#1AC69C]/50 text-[#59F3A6]"
                onClick={() =>
                  onApplyChange?.({
                    fileName: "",
                    startLine,
                    endLine,
                    suggestedCode,
                  })
                }
              >
                Apply
              </button>
              <button
                className="px-2 py-1 text-xs rounded-md bg-[#FF32C6]/10 border border-[#FF32C6]/50 text-[#FF32C6]"
                onClick={() =>
                  onRejectChange?.({ fileName: "", startLine, endLine })
                }
              >
                Reject
              </button>
            </div>
          </div>
          <pre className="bg-[#0D0D0D] rounded p-2 mb-2 text-sm overflow-x-auto">
            <code>{suggestedCode}</code>
          </pre>
          <p className="text-sm text-white/70">{explanation}</p>
        </div>
      );
    }

    return null;
  };

  return (
    <ResizablePanel
      direction="horizontal"
      defaultSize={400}
      minSize={300}
      maxSize={600}
      className={`border-l border-[#27272a] transition-transform duration-300 ${
        isOpen ? "translate-x-0" : "translate-x-full"
      }`}
      style={{
        position: "relative",
        height: "100%",
        flexShrink: 0,
        display: isOpen ? "flex" : "none",
      }}
    >
      <div className="h-full w-full flex flex-col bg-[#0D0D0D]">
        {/* Header */}
        <div className="flex-none flex items-center justify-between border-b border-[#27272a] p-4">
          <h2 className="text-xl font-medium">Chat</h2>
          <div className="flex space-x-2">
            <button
              className="p-1 text-white/50 hover:text-white"
              onClick={() => {
                setMessages([]);
                onClearAllRegions();
              }}
            >
              <PlusCircle size={20} />
            </button>
            <button className="p-1 text-white/50 hover:text-white">
              <Menu size={20} />
            </button>
            <button
              className="p-1 text-white/50 hover:text-white"
              onClick={onClose}
            >
              <X size={20} />
            </button>
          </div>
        </div>

        {/* Messages area */}
        <div className="flex-1 overflow-y-auto p-4 min-h-0">
          {messages.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-center">
              <Image
                src={"/auger-terminal.svg"}
                alt="Chat"
                width={100}
                height={100}
                className="w-48 h-48 mb-6"
              />
              <h3 className="text-2xl font-semibold mb-2">Ask a Question</h3>
              <p className="text-white/50 text-sm max-w-xs">
                Tag a memory region by highlighting and pressing{" "}
                <span className="bg-[#1A1A1A] px-1 rounded">⌘ + I</span>, or use{" "}
                <span className="bg-[#1A1A1A] px-1 rounded">⌘ + L</span> to
                toggle the chat window.
              </p>
            </div>
          ) : (
            <div className="space-y-4">
              {messages.map((message, index) => {
                const data = message.data as MessageData | undefined;
                return (
                  <div
                    key={index}
                    className={`p-3 rounded-lg ${
                      message.role === "user"
                        ? "bg-black/30 border border-white/10 text-[13px] ml-8"
                        : "bg-[#27272A]/50 border border-white/10 text-[13px] mr-8"
                    }`}
                  >
                    {data?.codeRegions && data.codeRegions.length > 0 && (
                      <div className="mb-2 flex flex-wrap gap-2">
                        {data.codeRegions.map((region: CodeRegion) => (
                          <div
                            key={region.id}
                            className="bg-[#1AC69C]/10 border border-[#1AC69C]/50 rounded-lg inline-block text-[#59F3A6] px-1 py-0.5 text-[10px]"
                          >
                            {formatMemoryAddress(region.start)} -{" "}
                            {formatMemoryAddress(region.end)}
                          </div>
                        ))}
                      </div>
                    )}
                    <div className="whitespace-break-spaces">
                      <ReactMarkdown
                        className={cn(
                          "prose prose-invert max-w-none prose-p:text-white/50 prose-p:mb-4 prose-p:mt-2 prose-pre:bg-zinc-800/50 prose-headings:font-bold prose-headings:mb-4 prose-headings:mt-6 prose-ul:mb-4 prose-ul:mt-2 prose-li:my-1 prose-li:pl-1 prose-pre:my-3 prose-code:px-1 prose-code:py-0.5 prose-code:bg-[#1A1A1A] prose-code:rounded prose-strong:font-bold prose-strong:text-white prose-em:text-white/90 prose-ul:list-disc prose-ol:list-decimal whitespace-break-spaces"
                        )}
                        remarkPlugins={[remarkGfm]}
                        components={{}}
                      >
                        {message.content}
                      </ReactMarkdown>
                    </div>
                    {data?.toolCalls?.map((toolCall: ToolCall, i: number) => (
                      <div key={i}>{renderToolCall(toolCall)}</div>
                    ))}
                  </div>
                );
              })}
              {isLoading && (
                <div className="flex items-center justify-start py-4">
                  <div className="animate-puls text-xs text-white/50">
                    Thinking...
                  </div>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Input area */}
        <div className="flex-none p-4">
          <div className="relative bg-[#1A1A1A] rounded-lg">
            {selectedRegions.length > 0 && (
              <div className="flex items-center justify-between gap-2 px-2 pt-2">
                <div className="flex items-center gap-2 w-full">
                  <div className="bg-white/10 hover:bg-white/20 rounded-full p-1">
                    <Plus size={12} className="text-white/50" />
                  </div>
                  <div className="flex-1 overflow-hidden">
                    <div className="bg-[#1AC69C]/10 border border-[#1AC69C]/50 rounded-lg px-1 py-0.5 text-[10px] text-[#59F3A6] flex items-center gap-2 w-fit">
                      <Tag size={10} />
                      {selectedRegions[activeTagIndex] &&
                        `${formatMemoryAddress(
                          selectedRegions[activeTagIndex].start
                        )} - ${formatMemoryAddress(
                          selectedRegions[activeTagIndex].end
                        )}`}
                      <button
                        className="text-[#59F3A6]/50 hover:text-[#59F3A6]"
                        onClick={() =>
                          selectedRegions[activeTagIndex] &&
                          onRemoveRegion(selectedRegions[activeTagIndex].id)
                        }
                      >
                        <X size={12} />
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            )}
            <textarea
              ref={textareaRef}
              className="w-full bg-transparent p-3 outline-none resize-none text-sm placeholder:text-white/30"
              placeholder="What do you see wrong with this code?"
              rows={2}
              value={input}
              onChange={handleInputChange}
              onKeyDown={handleKeyDown}
            />
            <button
              className="absolute bottom-2 right-2 p-1 rounded-md bg-[#1AC69C]/10 border border-[#1AC69C]/50 text-[#59F3A6] disabled:opacity-50"
              disabled={!input.trim()}
              onClick={handleSubmit}
            >
              <ArrowUp size={16} />
            </button>
          </div>
        </div>
      </div>
    </ResizablePanel>
  );
}
