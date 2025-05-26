import { Cog } from "lucide-react";

export default function CrossReferencePanel() {
  return (
    <div className="flex flex-col bg-[#0D0D0D]">
      <div className="p-4">
        <h2 className="text-xl font-medium">Cross Reference</h2>
        <p className="text-white/50 text-sm mt-1">
          Check a selection against other known instances.
        </p>
      </div>

      <div className="px-4 pb-4 flex-1 flex flex-col">
        <div className="bg-[#0D0D0D] rounded-lg border border-white/10 overflow-hidden flex-1 flex flex-col">
          <div className="grid grid-cols-2 text-xs text-white/50 p-2 border-b border-white/10">
            <div>Symbol</div>
            <div>Instances</div>
          </div>

          <div className="flex-1 flex items-center justify-center p-8">
            <div className="flex flex-col items-center text-center">
              <div className="p-2 rounded-lg bg-gradient-to-b from-[#1C1C1C] to-[#111111] border border-white/10 mb-4">
                <Cog className="w-6 h-6 text-white/10" />
              </div>
              <h3 className="text-lg font-medium mb-2">No Cross References</h3>
              <p className="text-white/30 text-xs max-w-[200px]">
                Select a symbol, type or region to populate cross references.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
