"use client";

import type React from "react";
import { useState, useEffect, useRef } from "react";
import type { CodeRegion } from "@/app/editor/page";
import { useKeyboardShortcut } from "@/hooks/use-keyboard-shortcut";

interface CodeEditorProps {
  code: string;
  selectedRegions: CodeRegion[];
  onAddRegion: (region: { start: number; end: number }) => void;
  onRemoveRegion: (regionId: string) => void;
  onSelectionChange?: (
    selection: { start: number; end: number } | null
  ) => void;
  pendingChanges?: Array<{
    fileName: string;
    startLine: number;
    endLine: number;
    suggestedCode: string;
  }>;
}

export default function CodeEditor({
  code,
  selectedRegions,
  onAddRegion,
  onRemoveRegion,
  onSelectionChange,
  pendingChanges = [],
}: CodeEditorProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [lines, setLines] = useState<string[]>([]);
  const [highlightedVariable, setHighlightedVariable] = useState<string | null>(
    null
  );
  const [functionLines, setFunctionLines] = useState<number[]>([]);
  const [selectionStart, setSelectionStart] = useState<number | null>(null);
  const [tempSelectedLines, setTempSelectedLines] = useState<number[]>([]);
  const [identifiers, setIdentifiers] = useState<{
    functions: Set<string>;
    variables: Set<string>;
    properties: Set<string>;
  }>({
    functions: new Set(),
    variables: new Set(),
    properties: new Set(),
  });

  // Add a new state for right-click context menu
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    visible: boolean;
    lineIndex?: number;
  }>({
    x: 0,
    y: 0,
    visible: false,
  });

  const editorRef = useRef<HTMLDivElement>(null);

  // Process the code on mount or when code changes
  useEffect(() => {
    // Split the code into lines
    const codeLines = code.split("\n");
    setLines(codeLines);

    // Find function declaration lines
    const fnLines: number[] = [];
    const foundFunctions = new Set<string>();
    const foundVariables = new Set<string>();
    const foundProperties = new Set<string>();

    // Process each line to identify functions, variables, and properties
    codeLines.forEach((line, index) => {
      // Detect function declarations
      if (
        line.trim().startsWith("fn ") &&
        line.includes("(") &&
        line.includes(")")
      ) {
        fnLines.push(index);

        // Extract function name
        const match = line.match(/fn\s+([a-zA-Z0-9_]+)\s*\(/);
        if (match && match[1]) {
          foundFunctions.add(match[1]);
        }
      }

      // Detect variable declarations
      const letMatches = line.matchAll(/let\s+([a-zA-Z0-9_]+)\s*=/g);
      for (const match of letMatches) {
        if (match[1]) foundVariables.add(match[1]);
      }

      // Detect property access
      const propMatches = line.matchAll(/\.([a-zA-Z0-9_]+)/g);
      for (const match of propMatches) {
        if (match[1]) foundProperties.add(match[1]);
      }

      // Detect function calls
      const funcCallMatches = line.matchAll(/([a-zA-Z0-9_]+)\s*\(/g);
      for (const match of funcCallMatches) {
        if (match[1] && !match[1].match(/^(if|for|while|switch)$/)) {
          foundFunctions.add(match[1]);
        }
      }
    });

    setFunctionLines(fnLines);
    setIdentifiers({
      functions: foundFunctions,
      variables: foundVariables,
      properties: foundProperties,
    });
  }, [code]);

  // Update parent component when selection changes
  useEffect(() => {
    if (tempSelectedLines.length > 0) {
      const start = Math.min(...tempSelectedLines);
      const end = Math.max(...tempSelectedLines);
      onSelectionChange?.({ start, end });
    } else {
      onSelectionChange?.(null);
    }
  }, [tempSelectedLines, onSelectionChange]);

  // Modify the handleLineClick function to update the selection style
  const handleLineClick = (lineIndex: number, event: React.MouseEvent) => {
    // If shift key is pressed and we have a selection start, create a range
    if (event.shiftKey && selectionStart !== null) {
      const start = Math.min(selectionStart, lineIndex);
      const end = Math.max(selectionStart, lineIndex);
      const range = Array.from(
        { length: end - start + 1 },
        (_, i) => start + i
      );
      setTempSelectedLines(range);
    } else {
      // Otherwise, start a new selection
      setSelectionStart(lineIndex);
      setTempSelectedLines([lineIndex]);

      // If we're clicking on a new line, clear the previous selection
      if (selectionStart !== lineIndex) {
        setSelectionStart(lineIndex);
      }
    }
  };

  // Function to handle tagging the current selection
  const handleTagSelection = () => {
    if (tempSelectedLines.length > 0) {
      const start = Math.min(...tempSelectedLines);
      const end = Math.max(...tempSelectedLines);
      onAddRegion({ start, end });
      setSelectionStart(null);
      setTempSelectedLines([]);
    }
  };

  // Add keyboard shortcut for tagging selection
  useKeyboardShortcut("i", handleTagSelection, { metaKey: true });

  // Find all occurrences of a variable
  const findVariableOccurrences = (variable: string): number[] => {
    const occurrences: number[] = [];

    // Simple regex to find word boundaries
    const regex = new RegExp(`\\b${variable}\\b`, "g");

    lines.forEach((line, index) => {
      if (regex.test(line)) {
        occurrences.push(index);
      }
    });

    return occurrences;
  };

  // Handle variable click
  const handleVariableClick = (variable: string) => {
    if (highlightedVariable === variable) {
      setHighlightedVariable(null);
    } else {
      setHighlightedVariable(variable);
    }
  };

  // Check if a line is in any of the selected regions
  const isLineSelected = (lineIndex: number): boolean => {
    return selectedRegions.some(
      (region) => lineIndex >= region.start && lineIndex <= region.end
    );
  };

  // Find which region a line belongs to
  const getRegionForLine = (lineIndex: number): CodeRegion | undefined => {
    return selectedRegions.find(
      (region) => lineIndex >= region.start && lineIndex <= region.end
    );
  };

  // Add a function to handle right-click
  const handleContextMenu = (e: React.MouseEvent, lineIndex: number) => {
    e.preventDefault();

    // Get the editor's position to calculate relative coordinates
    const editorRect = editorRef.current?.getBoundingClientRect();

    if (!editorRect) return;

    // Calculate position relative to the viewport
    const x = e.clientX;
    const y = e.clientY;

    setContextMenu({
      x,
      y,
      visible: true,
      lineIndex,
    });

    // If we're not in the middle of a selection, set this line as the temporary selection
    if (selectionStart === null) {
      setTempSelectedLines([lineIndex]);
    }
  };

  // Add a function to close the context menu
  const closeContextMenu = () => {
    setContextMenu((prev) => ({ ...prev, visible: false }));
    // Clear temporary selection if we're not in the middle of a selection
    if (selectionStart === null) {
      setTempSelectedLines([]);
    }
  };

  // Add a click handler to the document to close the context menu when clicking outside
  useEffect(() => {
    const handleDocumentClick = (e: MouseEvent) => {
      // Only close if the click is outside the context menu
      if (contextMenu.visible) {
        const target = e.target as Node;
        const contextMenuElement = document.getElementById("context-menu");
        if (contextMenuElement && !contextMenuElement.contains(target)) {
          closeContextMenu();
        }
      }
    };

    document.addEventListener("click", handleDocumentClick);
    return () => {
      document.removeEventListener("click", handleDocumentClick);
    };
  }, [contextMenu.visible]);

  // Add a function to add the selected region to chat
  const addToChat = () => {
    if (contextMenu.lineIndex !== undefined) {
      // If we have a temporary selection, use that
      if (tempSelectedLines.length > 0) {
        const start = Math.min(...tempSelectedLines);
        const end = Math.max(...tempSelectedLines);
        onAddRegion({ start, end });
      } else {
        // Otherwise, use the line that was right-clicked
        onAddRegion({
          start: contextMenu.lineIndex,
          end: contextMenu.lineIndex,
        });
      }
    }
    closeContextMenu();
    // Reset selection state
    setSelectionStart(null);
    setTempSelectedLines([]);
  };

  // Add a function to deselect a specific region
  const deselectRegion = () => {
    if (contextMenu.lineIndex !== undefined) {
      const region = getRegionForLine(contextMenu.lineIndex);
      if (region) {
        onRemoveRegion(region.id);
      }
    }
    closeContextMenu();
  };

  // Create React elements directly instead of using dangerouslySetInnerHTML
  const renderLine = (line: string, lineIndex: number) => {
    // Check if this line is part of a pending change
    const pendingChange = pendingChanges.find(
      (change) => lineIndex >= change.startLine && lineIndex <= change.endLine
    );

    if (pendingChange) {
      return (
        <span className="bg-[#1AC69C]/10 border-l-2 border-[#1AC69C] pl-2">
          {line}
        </span>
      );
    }

    if (!line.trim()) return <span>&nbsp;</span>;

    // Create an array to hold the parts of the line
    const parts: React.ReactNode[] = [];
    let currentIndex = 0;
    let key = 0;

    // Helper function to add a plain text segment
    const addText = (end: number) => {
      if (end > currentIndex) {
        parts.push(
          <span key={key++}>{line.substring(currentIndex, end)}</span>
        );
        currentIndex = end;
      }
    };

    // Process keywords
    const keywords = [
      "fn",
      "let",
      "if",
      "for",
      "return",
      "while",
      "match",
      "struct",
      "enum",
      "impl",
    ];
    keywords.forEach((keyword) => {
      const regex = new RegExp(`\\b${keyword}\\b`, "g");
      let match;
      while ((match = regex.exec(line)) !== null) {
        addText(match.index);
        parts.push(
          <span key={key++} className="text-[#C478FF]">
            {keyword}
          </span>
        );
        currentIndex = match.index + keyword.length;
      }
    });

    // Process special values
    const specialValues = ["None", "false", "true", "null", "undefined"];
    specialValues.forEach((value) => {
      const regex = new RegExp(`\\b${value}\\b`, "g");
      let match;
      while ((match = regex.exec(line)) !== null) {
        addText(match.index);
        parts.push(
          <span key={key++} className="text-[#FF6B6B]">
            {value}
          </span>
        );
        currentIndex = match.index + value.length;
      }
    });

    // Process strings
    const stringRegex = /"([^"\\]*(\\.[^"\\]*)*)"|'([^'\\]*(\\.[^'\\]*)*)'/g;
    let stringMatch;
    while ((stringMatch = stringRegex.exec(line)) !== null) {
      addText(stringMatch.index);
      parts.push(
        <span key={key++} className="text-[#59F3A6]">
          {stringMatch[0]}
        </span>
      );
      currentIndex = stringMatch.index + stringMatch[0].length;
    }

    // Process comments
    const commentRegex = /\/\/(.*$)/;
    const commentMatch = commentRegex.exec(line);
    if (commentMatch) {
      addText(commentMatch.index);
      parts.push(
        <span key={key++} className="text-gray-500">
          {commentMatch[0]}
        </span>
      );
      currentIndex = line.length;
    }

    // Process function names (dynamically identified)
    for (const funcName of identifiers.functions) {
      const regex = new RegExp(`\\b${funcName}\\b(?!\\()`, "g");
      let match;
      while ((match = regex.exec(line)) !== null) {
        addText(match.index);
        parts.push(
          <span key={key++} className="text-[#FF32C6]">
            {funcName}
          </span>
        );
        currentIndex = match.index + funcName.length;
      }
    }

    // Process properties (dynamically identified)
    for (const propName of identifiers.properties) {
      const regex = new RegExp(`\\.${propName}\\b`, "g");
      let match;
      while ((match = regex.exec(line)) !== null) {
        addText(match.index);
        parts.push(<span key={key++}>.</span>);
        parts.push(
          <span key={key++} className="text-[#4D9EFF]">
            {propName}
          </span>
        );
        currentIndex = match.index + propName.length + 1; // +1 for the dot
      }
    }

    // Process variables (dynamically identified)
    for (const varName of identifiers.variables) {
      const regex = new RegExp(`\\b${varName}\\b`, "g");
      let match;
      while ((match = regex.exec(line)) !== null) {
        addText(match.index);

        // Check if this variable is currently highlighted
        const isHighlighted = highlightedVariable === varName;

        parts.push(
          <span
            key={key++}
            className={`${
              isHighlighted ? "bg-[#3A3A3A] rounded px-1" : ""
            } text-[#9580FF]`}
            onClick={(e) => {
              e.stopPropagation();
              handleVariableClick(varName);
            }}
          >
            {varName}
          </span>
        );
        currentIndex = match.index + varName.length;
      }
    }

    // Add any remaining text
    addText(line.length);

    return parts.length > 0 ? parts : <span>{line}</span>;
  };

  // Variable occurrences
  const variableOccurrences = highlightedVariable
    ? findVariableOccurrences(highlightedVariable)
    : [];

  // Format a memory address from a line number
  const formatMemoryAddress = (lineNum: number): string => {
    return `0x${lineNum.toString(16).padStart(4, "0").toUpperCase()}`;
  };

  // Update the return statement to include the context menu
  return (
    <div
      ref={editorRef}
      className="bg-[#0D0D0D] p-4 font-mono text-sm relative"
      onClick={() => {
        // Only close context menu if we click outside of it
        if (contextMenu.visible) {
          closeContextMenu();
        }
        // Reset selection if we click in the editor but not on a line
        setSelectionStart(null);
        setTempSelectedLines([]);
      }}
    >
      <table className="w-full border-collapse">
        <tbody>
          {lines.map((line, i) => {
            const lineNumber = i.toString().padStart(8, "0");

            const isFunctionLine = functionLines.includes(i);
            const isVariableHighlightLine = variableOccurrences.includes(i);
            const isPermanentlySelected = isLineSelected(i);
            const isTemporarilySelected = tempSelectedLines.includes(i);

            return (
              <tr
                key={i}
                className={`
                  ${
                    isFunctionLine
                      ? "bg-[#1A1A1A] border-t border-b border-[#333333]"
                      : ""
                  } 
                  ${
                    isVariableHighlightLine && !isFunctionLine
                      ? "bg-[#1A1A1A]/50"
                      : ""
                  }
                  ${isPermanentlySelected ? "bg-[#FF32C6]/10" : ""}
                  ${
                    isTemporarilySelected && !isPermanentlySelected
                      ? "bg-[#333333]"
                      : ""
                  }
                  cursor-pointer
                `}
                onClick={(e) => {
                  e.stopPropagation();
                  handleLineClick(i, e);
                }}
                onContextMenu={(e) => {
                  e.stopPropagation();
                  handleContextMenu(e, i);
                }}
              >
                <td
                  className={`select-none text-white/20 pr-4 pl-4 text-right whitespace-nowrap align-top ${
                    isFunctionLine ? "mt-1 py-2" : "py-0"
                  }`}
                >
                  {lineNumber}
                </td>
                <td className="w-full whitespace-pre">
                  <div className={`${isFunctionLine ? "py-2" : "py-0"}`}>
                    {renderLine(line, i)}
                  </div>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>

      {contextMenu.visible && (
        <div
          id="context-menu"
          className="fixed z-50 bg-[#1A1A1A] border border-[#333333] rounded-md shadow-lg py-1 min-w-[180px]"
          style={{
            left: contextMenu.x,
            top: contextMenu.y,
            maxWidth: "250px",
          }}
          onClick={(e) => e.stopPropagation()}
        >
          {contextMenu.lineIndex !== undefined &&
          isLineSelected(contextMenu.lineIndex) ? (
            <button
              className="w-full text-left px-3 py-2 flex items-center gap-2 hover:bg-[#333333] text-sm"
              onClick={deselectRegion}
            >
              <span className="text-white/70">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d="M3 6h18"></path>
                  <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
                  <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path>
                </svg>
              </span>
              <span className="flex-1">Deselect</span>
              <span className="text-white/50 text-xs bg-[#0D0D0D] px-1.5 py-0.5 rounded">
                ⌘ P
              </span>
            </button>
          ) : (
            <button
              className="w-full text-left px-3 py-2 flex items-center gap-2 hover:bg-[#333333] text-sm"
              onClick={addToChat}
            >
              <span className="text-white/70">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path>
                  <line x1="7" y1="7" x2="7.01" y2="7"></line>
                </svg>
              </span>
              <span className="flex-1">Add To Chat</span>
              <span className="text-white/50 text-xs bg-[#0D0D0D] px-1.5 py-0.5 rounded">
                ⌘ L
              </span>
            </button>
          )}
          <button className="w-full text-left px-3 py-2 flex items-center gap-2 hover:bg-[#333333] text-sm">
            <span className="text-white/70">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
              </svg>
            </span>
            <span className="flex-1">Tools</span>
            <span className="text-white/50">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <polyline points="9 18 15 12 9 6"></polyline>
              </svg>
            </span>
          </button>
        </div>
      )}
    </div>
  );
}
