"use client";

import { useCallback, useState } from "react";
import Layout from "@/components/layout";
import CodeEditor from "@/components/code-editor";
import SymbolsPanel from "@/components/symbols-panel";
import CrossReferencePanel from "@/components/cross-reference-panel";
import ChatPanel from "@/components/chat-panel";
import ResizablePanel from "@/components/resizable-panel";
import EditorFooter from "@/components/editor-footer";
import { useKeyboardShortcut } from "@/hooks/use-keyboard-shortcut";

export interface CodeRegion {
  id: string;
  start: number;
  end: number;
  code: string;
}

interface SuggestedChange {
  fileName: string;
  startLine: number;
  endLine: number;
  suggestedCode: string;
}

export default function EditorPage() {
  const [tabs, setTabs] = useState([
    { id: "tab1", label: "my_awesome_project", active: true },
    { id: "tab2", label: "another_project", active: false },
  ]);

  const [activeTab, setActiveTab] = useState("Psuedo Rust");
  const [isChatOpen, setIsChatOpen] = useState(false);
  const [selectedCodeRegions, setSelectedCodeRegions] = useState<CodeRegion[]>(
    []
  );
  const [currentSelection, setCurrentSelection] = useState<{
    start: number;
    end: number;
  } | null>(null);
  const [sampleCode, setSampleCode] = useState(`// Pseudo Rust Code ->

fn updateGutters(cm) {
  let gutters = cm.display.gutters,
      __specs = cm.options.gutters;
      
  removeChildren(gutters);
  
  for i specs.len() {
    let gutterClass = __specs[i];
    let gElt = gutters.appendChild(
      elt(
        "div",
        None,
        "CodeMirror-gutter " + gutterClass
      )
    );
    if gutterClass == "CodeMirror-linenumbers" {
      cm.display.lineGutter = gElt;
      gElt.style.width;
    }
  }
  gutters.style.display = i.unwrap_or("none");
  updateGutterSpace(cm);
  
  return false;
}

fn doSomething() {
  println!("We're doing something...");
  let catch = vec![];
}`);
  const [pendingChanges, setPendingChanges] = useState<SuggestedChange[]>([]);
  const editorTabs = ["Psuedo Rust", "sBPF Assembly"];

  // Toggle chat with keyboard shortcut (Command + L)
  useKeyboardShortcut(
    "l",
    () => {
      setIsChatOpen((prev) => !prev);
      // If we have a current selection, add it as a region when opening chat
      if (!isChatOpen && currentSelection) {
        addRegion(currentSelection);
      }
    },
    { metaKey: true }
  );

  // Add keyboard shortcut for Command+I to tag selection
  useKeyboardShortcut(
    "i",
    () => {
      if (currentSelection) {
        addRegion(currentSelection);
      }
    },
    { metaKey: true }
  );

  const handleTabChange = (tabId: string) => {
    setTabs(
      tabs.map((tab) => ({
        ...tab,
        active: tab.id === tabId,
      }))
    );
  };

  const handleTabClose = (tabId: string) => {
    const newTabs = tabs.filter((tab) => tab.id !== tabId);
    if (tabs.find((tab) => tab.id === tabId)?.active && newTabs.length > 0) {
      newTabs[0].active = true;
    }
    setTabs(newTabs);
  };

  const toggleChat = () => {
    setIsChatOpen((prev) => !prev);
  };

  const addRegion = useCallback(
    (region: { start: number; end: number }) => {
      const newRegion: CodeRegion = {
        id: `region-${Date.now()}`,
        start: region.start,
        end: region.end,
        code: sampleCode
          .split("\n")
          .slice(region.start, region.end + 1)
          .join("\n"),
      };

      setSelectedCodeRegions((prev) => [...prev, newRegion]);
    },
    [sampleCode]
  );

  const removeRegion = useCallback((regionId: string) => {
    setSelectedCodeRegions((prev) =>
      prev.filter((region) => region.id !== regionId)
    );
  }, []);

  const handleSelectionChange = useCallback(
    (selection: { start: number; end: number } | null) => {
      setCurrentSelection(selection);
    },
    []
  );

  // Clear all selected regions
  const clearAllRegions = useCallback(() => {
    setSelectedCodeRegions([]);
  }, []);

  // Handle message sent - clear all regions
  const handleMessageSent = useCallback(() => {
    clearAllRegions();
  }, [clearAllRegions]);

  // Handle applying a code change
  const handleApplyChange = useCallback(
    (change: SuggestedChange) => {
      // Apply the change to the code
      const codeLines = sampleCode.split("\n");
      const beforeChange = codeLines.slice(0, change.startLine).join("\n");
      const afterChange = codeLines.slice(change.endLine + 1).join("\n");
      const newCode = `${beforeChange}\n${change.suggestedCode}\n${afterChange}`;

      // Update the code
      setSampleCode(newCode);

      // Remove the change from pending changes
      setPendingChanges((prev) =>
        prev.filter(
          (c) =>
            !(
              c.startLine === change.startLine &&
              c.endLine === change.endLine &&
              c.fileName === change.fileName
            )
        )
      );
    },
    [sampleCode]
  );

  // Handle rejecting a code change
  const handleRejectChange = useCallback(
    (change: Omit<SuggestedChange, "suggestedCode">) => {
      // Remove the change from pending changes
      setPendingChanges((prev) =>
        prev.filter(
          (c) =>
            !(
              c.startLine === change.startLine &&
              c.endLine === change.endLine &&
              c.fileName === change.fileName
            )
        )
      );
    },
    []
  );

  return (
    <Layout
      tabs={tabs}
      title="Editor"
      onTabChange={handleTabChange}
      onTabClose={handleTabClose}
      showEditorIcons={true}
      onChatToggle={toggleChat}
      isChatOpen={isChatOpen}
    >
      <div className="flex flex-col h-[calc(100vh-2.5rem)]">
        <div className="flex flex-1 overflow-hidden">
          {/* Left panel with Symbols and Cross References */}
          <ResizablePanel
            direction="horizontal"
            defaultSize={300}
            maxSize={500}
            className="border-r border-[#27272a]"
          >
            <ResizablePanel
              direction="vertical"
              defaultSize={300}
              minSize={150}
              className="border-b border-[#27272a]"
            >
              <SymbolsPanel />
            </ResizablePanel>
            <CrossReferencePanel />
          </ResizablePanel>

          {/* Main editor area - will shrink when chat is open */}
          <div className="flex-1 flex flex-col min-w-0">
            {/* Editor tabs */}
            <div className="flex bg-[#0D0D0D] border-b border-[#27272a]">
              {editorTabs.map((tab) => (
                <button
                  key={tab}
                  className={`px-4 py-2 text-sm ${
                    activeTab === tab
                      ? "bg-[#1C1C1C] text-white border-t border-l border-r border-[#27272a]"
                      : "text-white/40"
                  }`}
                  onClick={() => setActiveTab(tab)}
                >
                  {tab}
                </button>
              ))}
            </div>

            {/* Code editor */}
            <div className="flex-1 overflow-auto">
              <CodeEditor
                code={sampleCode}
                selectedRegions={selectedCodeRegions}
                onAddRegion={addRegion}
                onRemoveRegion={removeRegion}
                onSelectionChange={handleSelectionChange}
                pendingChanges={pendingChanges}
              />
            </div>
          </div>

          {/* Chat panel - when open, it takes space from the editor */}
          <ChatPanel
            isOpen={isChatOpen}
            onClose={() => setIsChatOpen(false)}
            selectedRegions={selectedCodeRegions}
            onRemoveRegion={removeRegion}
            onClearAllRegions={clearAllRegions}
            onMessageSent={handleMessageSent}
            onApplyChange={handleApplyChange}
            onRejectChange={handleRejectChange}
          />
        </div>

        {/* Footer */}
        <EditorFooter />
      </div>
    </Layout>
  );
}
