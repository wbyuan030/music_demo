import { useState } from "react";
import { useSearchStore } from "../store/Search";
import { useStateStore } from "../store/State";
import { StateEnum } from "../types/state";
import { Search } from "lucide-react"
function GoogleSearchInput() {
  const search = useSearchStore((state) => state.search)
  const isLoading = useSearchStore((state) => state.isLoading)
  const setState = useStateStore((state) => state.setCurrentState)
  const [searchText, setSearchText] = useState("")
  return (
    <div className="w-full max-w-2xl mx-auto flex flex-row gap-2"> {/* 外层容器，控制最大宽度 */}
      <input
        id="input"
        className="truncate flex-1 h-12 px-6 rounded-full border border-gray-200 bg-white text-gray-700 shadow-sm hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-shadow"
        type="search"
        autoComplete="on"
        spellCheck={false}
        onChange={(e) => setSearchText(e.target.value)}

        role="combobox"
        aria-controls="matches"
        aria-live="polite"
        aria-expanded="false"
        aria-description=""
        placeholder="搜索音乐"
      />
      <button
        onClick={() => {
          search(searchText)
          setState(StateEnum.searchResult)
        }}
        className="h-12 px-6 bg-blue-500  rounded-full hover:bg-blue-600 transition-colors shadow-sm font-medium whitespace-nowrap  disabled:hidden"
        disabled={isLoading}
      ><Search /></button>
    </div>
  );
}

export default GoogleSearchInput;
