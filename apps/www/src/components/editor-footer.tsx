import { Anchor, Braces, Brackets, Component } from "lucide-react";
import Image from "next/image";

export default function EditorFooter() {
  return (
    <div className="h-6 bg-[#0D0D0D] border-t border-white/10 flex items-center justify-between px-4 text-xs text-white/50">
      <div className="flex items-center space-x-4">
        <span className="flex items-center gap-1">
          <Braces size={12} />
          0×0138-0×0150
        </span>
        <span className="flex items-center gap-1">
          <Component size={12} />
          SVM Analyzer
        </span>
        <span className="flex items-center gap-1">
          <Component size={12} />
          sBPF Parser
        </span>
      </div>
      <div className="flex items-center space-x-4">
        <span className="flex items-center gap-1">
          <Anchor size={12} />
          Anchor
        </span>
        <div>
          <Image src="/icons/solana.svg" alt="Solana" width={14} height={14} />
        </div>
      </div>
    </div>
  );
}
