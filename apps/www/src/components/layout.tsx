"use client";

import type React from "react";

import { useState } from "react";
import Link from "next/link";
import Image from "next/image";
import { TabBar } from "@/components/tab-bar";
import { Home, Folder, Box, FileText } from "lucide-react";

interface LayoutProps {
  children: React.ReactNode;
  tabs: {
    id: string;
    label: string;
    active?: boolean;
  }[];
  title: string;
  onTabChange?: (tabId: string) => void;
  onTabClose?: (tabId: string) => void;
  showEditorIcons?: boolean;
  onChatToggle?: () => void;
  isChatOpen?: boolean;
}

export default function Layout({
  children,
  tabs,
  title,
  onTabChange,
  onTabClose,
  showEditorIcons = false,
  onChatToggle,
  isChatOpen = false,
}: LayoutProps) {
  const [openTabs, setOpenTabs] = useState([
    { id: "my_awesome_project", label: "my_awesome_project", active: true },
    { id: "another_project", label: "another_project", active: false },
  ]);

  const closeTab = (id: string) => {
    setOpenTabs(openTabs.filter((tab) => tab.id !== id));
  };

  const activateTab = (id: string) => {
    setOpenTabs(
      openTabs.map((tab) => ({
        ...tab,
        active: tab.id === id,
      }))
    );
  };

  return (
    <div className="min-h-screen flex bg-[#0D0D0D] text-white">
      <aside className="w-16 bg-[#0D0D0D] border-r border-[#27272a] flex flex-col items-center space-y-8 pb-4">
        <div>
          <div className="w-14 h-14 flex items-center justify-center">
            <Image
              src="/auger-logomark.svg"
              alt="Auger Logo"
              width={40}
              height={40}
              priority
            />
          </div>
        </div>
        <div className="flex flex-col space-y-3">
          <Link href="/" className="p-2 hover:bg-[#FF0000]/30 rounded">
            <Image
              src="/icons/home.svg"
              alt="Home"
              width={20}
              height={20}
              className="text-white/40"
            />
          </Link>
          <Link href="/projects" className="p-2 hover:bg-[#FF0000]/30 rounded">
            <Image
              src="/icons/folder.svg"
              alt="Folder"
              width={20}
              height={20}
              className="text-white/40"
            />
          </Link>
          <Link
            href="/components"
            className="p-2 hover:bg-[#FF0000]/30 rounded"
          >
            <Image
              src="/icons/grid-plus.svg"
              alt="Grid Plus"
              width={20}
              height={20}
              className="text-white/40"
            />
          </Link>
          <Link href="/files" className="p-2 hover:bg-[#FF0000]/30 rounded">
            <Image
              src="/icons/db.svg"
              alt="Saved"
              width={20}
              height={20}
              className="text-white/40"
            />
          </Link>
        </div>
      </aside>

      <div className="w-full flex-col overflow-hidden">
        <header className="flex items-center h-10 border-b border-[#27272a] px-2">
          <TabBar
            tabs={tabs}
            title={title}
            onTabChange={onTabChange}
            onTabClose={onTabClose}
            showEditorIcons={showEditorIcons}
            onChatToggle={onChatToggle}
            isChatOpen={isChatOpen}
          />
        </header>

        <main className="flex-1 overflow-auto">{children}</main>
      </div>
    </div>
  );
}
