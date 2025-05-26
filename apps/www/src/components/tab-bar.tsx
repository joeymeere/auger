"use client";
import {
  ChevronLeft,
  ChevronRight,
  Settings,
  X,
  Component,
  MessageCircle,
} from "lucide-react";
import { cn } from "@/lib/utils";

interface Tab {
  id: string;
  label: string;
  active?: boolean;
}

interface TabBarProps {
  title: string;
  tabs: Tab[];
  onTabChange?: (tabId: string) => void;
  onTabClose?: (tabId: string) => void;
  showEditorIcons?: boolean;
  onChatToggle?: () => void;
  isChatOpen?: boolean;
}

export function TabBar({
  title,
  tabs,
  onTabChange,
  onTabClose,
  showEditorIcons = false,
  onChatToggle,
  isChatOpen = false,
}: TabBarProps) {
  return (
    <div className="flex h-10 w-full text-gray-300 text-sm">
      <div className="flex items-center mr-2">
        <button className="px-2 text-white/50 hover:text-white">
          <ChevronLeft className="h-4 w-4" />
        </button>
        <button className="px-2 text-white/50 hover:text-white">
          <ChevronRight className="h-4 w-4" />
        </button>
      </div>

      <div className="flex flex-1 overflow-x-auto">
        {tabs.map((tab) => (
          <TabItem
            key={tab.id}
            id={tab.id}
            label={tab.label}
            active={tab.active}
            onActivate={() => onTabChange?.(tab.id)}
            onClose={() => onTabClose?.(tab.id)}
          />
        ))}
      </div>

      {showEditorIcons && (
        <div className="flex items-center">
          <button className="px-2 text-white/50 hover:text-white">
            <Component className="h-5 w-5" strokeWidth={1} />
          </button>
          <button
            className={`px-2 ${
              isChatOpen ? "text-white" : "text-white/50"
            } hover:text-white`}
            onClick={onChatToggle}
          >
            <MessageCircle className="h-5 w-5" strokeWidth={1} />
          </button>
        </div>
      )}

      <button className="pr-4 pl-2 text-white/50 hover:text-white">
        <Settings strokeWidth={1} className="h-5 w-5" />
      </button>
    </div>
  );
}

interface TabItemProps {
  id: string;
  label: string;
  active?: boolean;
  onActivate?: () => void;
  onClose?: () => void;
}

function TabItem({ id, label, active, onActivate, onClose }: TabItemProps) {
  return (
    <div
      className={cn(
        "group relative flex h-10 items-center px-3 cursor-pointer",
        "before:absolute before:inset-0 before:rounded-t-md before:border-t before:border-l before:border-r before:border-transparent",
        active
          ? "text-white before:bg-zinc-900 before:border-white/10"
          : "hover:text-white before:hover:bg-zinc-900/75"
      )}
      onClick={onActivate}
    >
      <span className="relative z-10">{label}</span>
      <button
        className="relative z-10 ml-2 p-0.5 rounded-full opacity-60 hover:opacity-100 hover:bg-zinc-700"
        onClick={(e) => {
          e.stopPropagation();
          onClose?.();
        }}
      >
        <X className="h-3.5 w-3.5" />
      </button>
    </div>
  );
}
