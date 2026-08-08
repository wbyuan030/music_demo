import { useState } from "react";
import { useSearchStore } from "../store/Search";
import { useStateStore } from "../store/State";
import { StateEnum } from "../types/state";
import { Search } from "lucide-react"

const SOURCES = [
  { value: 'all', label: '全部' },
  { value: 'youtube', label: 'YouTube' },
  { value: 'bilibili', label: 'B站' },
] as const

function SearchInput() {
  const search = useSearchStore((state) => state.search)
  const isLoading = useSearchStore((state) => state.isLoading)
  const source = useSearchStore((state) => state.source)
  const setSource = useSearchStore((state) => state.setSource)
  const setState = useStateStore((state) => state.setCurrentState)
  const [searchText, setSearchText] = useState("")

  return (
    <div className="w-full max-w-2xl mx-auto flex flex-col gap-2.5">
      <div className="flex flex-row gap-2">
        <input
          id="input"
          className="truncate flex-1 h-11 px-5 rounded-full border border-neutral-800 bg-neutral-900 text-gray-200 placeholder:text-neutral-500 shadow-inner transition-all focus:outline-none focus:ring-2 focus:ring-emerald-500/50 focus:border-emerald-500/60"
          type="search"
          autoComplete="on"
          spellCheck={false}
          onChange={(e) => setSearchText(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && searchText.length > 0) {
              setState(StateEnum.searchResult)
              search(searchText)
            }
          }}
          role="combobox"
          aria-controls="matches"
          aria-live="polite"
          aria-expanded="false"
          placeholder="搜索音乐"
        />
        <button
          onClick={() => {
            setState(StateEnum.searchResult)
            search(searchText)
          }}
          className="h-11 px-6 rounded-full bg-emerald-500 hover:bg-emerald-400 active:scale-95 text-neutral-950 font-semibold transition-all shadow-lg shadow-emerald-500/20 disabled:opacity-40 disabled:pointer-events-none"
          disabled={isLoading || searchText.length == 0}
        ><Search /></button>
      </div>
      <div className="flex flex-row gap-2 justify-center">
        {SOURCES.map((s) => (
          <button
            key={s.value}
            onClick={() => setSource(s.value)}
            className={`px-3.5 py-1 rounded-full text-sm font-medium transition-all active:scale-95 ${
              source === s.value
                ? 'bg-emerald-500 text-neutral-950 shadow shadow-emerald-500/30'
                : 'bg-neutral-800/80 text-neutral-400 hover:bg-neutral-700 hover:text-neutral-200'
            }`}
          >
            {s.label}
          </button>
        ))}
      </div>
    </div>
  );
}

export default SearchInput;
