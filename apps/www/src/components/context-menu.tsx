"use client";

import { Copy, Tag, Trash, X } from "lucide-react";
import { useEffect, useRef } from "react";

interface ContextMenuOption {
  label: string;
  icon: "copy" | "tag" | "untag" | "trash";
  onClick: () => void;
  disabled?: boolean;
}

interface ContextMenuProps {
  x: number;
  y: number;
  options: ContextMenuOption[];
}

export function ContextMenu({ x, y, options }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Adjust position if menu would go off screen
    if (menuRef.current) {
      const rect = menuRef.current.getBoundingClientRect();
      const windowWidth = window.innerWidth;
      const windowHeight = window.innerHeight;

      let adjustedX = x;
      let adjustedY = y;

      if (x + rect.width > windowWidth) {
        adjustedX = windowWidth - rect.width - 10;
      }

      if (y + rect.height > windowHeight) {
        adjustedY = windowHeight - rect.height - 10;
      }

      menuRef.current.style.left = `${adjustedX}px`;
      menuRef.current.style.top = `${adjustedY}px`;
    }
  }, [x, y]);

  const getIcon = (iconName: string) => {
    switch (iconName) {
      case "copy":
        return <Copy size={16} />;
      case "tag":
        return <Tag size={16} />;
      case "untag":
        return <X size={16} />;
      case "trash":
        return <Trash size={16} />;
      default:
        return null;
    }
  };

  return (
    <div
      ref={menuRef}
      className="fixed z-50 bg-[#1A1A1A] border border-[#333333] rounded-md shadow-lg py-1 min-w-[180px]"
      style={{ left: x, top: y }}
      onClick={(e) => e.stopPropagation()}
    >
      {options.map((option, index) => (
        <button
          key={index}
          className={`w-full text-left px-3 py-2 flex items-center gap-2 hover:bg-[#333333] text-sm ${
            option.disabled ? "opacity-50 cursor-not-allowed" : ""
          }`}
          onClick={(e) => {
            e.stopPropagation();
            if (!option.disabled) {
              option.onClick();
            }
          }}
          disabled={option.disabled}
        >
          <span className="text-white/70">{getIcon(option.icon)}</span>
          {option.label}
        </button>
      ))}
    </div>
  );
}
