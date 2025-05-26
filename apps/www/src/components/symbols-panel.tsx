export default function SymbolsPanel() {
  const symbols = [
    { name: "println", address: "0x0138", kind: "macro" },
    { name: "updateGutters", address: "0x0138", kind: "fn" },
    { name: "gutters", address: "0x0138", kind: "Vec" },
    { name: "remove_children", address: "0x0138", kind: "fn" },
  ];

  return (
    <div className="h-full flex flex-col bg-[#0D0D0D]">
      <div className="p-4">
        <h2 className="text-xl font-medium">Symbols</h2>
        <p className="text-white/50 text-sm mt-1">
          View symbols included within the binary.
        </p>
      </div>

      <div className="px-4 pb-4">
        <div className="bg-[#0D0D0D] rounded-lg border border-white/10 overflow-hidden">
          <div className="grid grid-cols-3 text-xs text-white/50 p-2 border-b border-white/10">
            <div>Symbol</div>
            <div>Address</div>
            <div>Kind</div>
          </div>

          <div className="max-h-[calc(100vh-300px)] overflow-auto">
            {symbols.map((symbol, index) => (
              <div
                key={index}
                className="grid grid-cols-3 text-sm p-2 hover:bg-white/5 cursor-pointer"
              >
                <div className="text-[#FF32C6]">{symbol.name}</div>
                <div className="text-[#59F3A6]">{symbol.address}</div>
                <div
                  className={
                    symbol.kind === "fn"
                      ? "text-[#C478FF]"
                      : symbol.kind === "macro"
                      ? "text-[#FFAC60]"
                      : "text-white"
                  }
                >
                  {symbol.kind}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
