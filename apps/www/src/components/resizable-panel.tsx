"use client";

import type React from "react";

import { useState, useRef, useEffect } from "react";

interface ResizablePanelProps {
  children: React.ReactNode;
  direction: "horizontal" | "vertical";
  defaultSize: number;
  minSize?: number;
  maxSize?: number;
  className?: string;
  style?: React.CSSProperties;
}

export default function ResizablePanel({
  children,
  direction,
  defaultSize,
  minSize = 100,
  maxSize = 800,
  className = "",
  style,
}: ResizablePanelProps) {
  const [size, setSize] = useState(defaultSize);
  const [isResizing, setIsResizing] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);
  const startPosRef = useRef<number>(0);
  const startSizeRef = useRef<number>(defaultSize);

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    startPosRef.current = direction === "horizontal" ? e.clientX : e.clientY;
    startSizeRef.current = size;
    setIsResizing(true);
  };

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isResizing) return;

      const currentPos = direction === "horizontal" ? e.clientX : e.clientY;
      const delta = currentPos - startPosRef.current;
      const newSize = Math.max(
        minSize,
        Math.min(maxSize, startSizeRef.current + delta)
      );

      setSize(newSize);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
    };

    if (isResizing) {
      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
    }

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isResizing, direction, minSize, maxSize]);

  // Merge any custom styles passed in with the component's styles
  const combinedStyle = {
    [direction === "horizontal" ? "width" : "height"]: `${size}px`,
    flexShrink: 0,
    ...style,
  };

  return (
    <div
      ref={panelRef}
      className={`relative ${className}`}
      style={combinedStyle}
    >
      {children}

      <div
        className={`absolute ${
          direction === "horizontal"
            ? "right-0 top-0 bottom-0 w-1 cursor-col-resize"
            : "bottom-0 left-0 right-0 h-1 cursor-row-resize"
        } bg-transparent hover:bg-[#FF32C6]/30 z-10`}
        onMouseDown={handleMouseDown}
      />
    </div>
  );
}
