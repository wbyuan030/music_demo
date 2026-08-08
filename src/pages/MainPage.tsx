import { useState } from "react"
import MiniPlayer from "../components/MiniPlayer"
import MainLayout from "../layout/MainLayout"
import { ChevronLeft, Link, Search } from "lucide-react"
import SearchInput from "../components/SearchBar"
import ParseUrl from "../components/ParseUrl"
import MainPageContent from "../components/MainPageContent"

const iconButton = "group transition-all duration-200 hover:scale-105 active:scale-95"

const OriginBar = ({ setBarState }: { setBarState: Function }) => (
  <div className="flex w-full h-full px-6 items-center justify-between bg-neutral-900/70 border-b border-neutral-800 backdrop-blur-md">
    <button className={iconButton} onClick={() => setBarState("search")}>
      <Search className="text-neutral-400 group-hover:text-emerald-400 transition-colors" />
    </button>
    <span className="text-sm font-semibold tracking-widest uppercase text-neutral-300 select-none">
      Menu
    </span>
    <button className={iconButton} onClick={() => setBarState("parse")}>
      <Link className="text-neutral-400 group-hover:text-emerald-400 transition-colors" />
    </button>
  </div>
)

const BackButton = ({ onClick }: { onClick: () => void }) => (
  <button
    onClick={onClick}
    className="w-10 h-10 flex items-center justify-center rounded-full text-neutral-400 hover:text-white hover:bg-neutral-800/80 transition-colors"
  >
    <ChevronLeft />
  </button>
)

const SearchTopBar = ({ setBarState }: { setBarState: Function }) => (
  <div className="flex items-center gap-2 h-full px-4 bg-neutral-900/70 border-b border-neutral-800 backdrop-blur-md">
    <BackButton onClick={() => setBarState("origin")} />
    <SearchInput />
  </div>
)

const ParseTopBar = ({ setBarState }: { setBarState: Function }) => (
  <div className="flex items-center gap-2 h-full px-4 bg-neutral-900/70 border-b border-neutral-800 backdrop-blur-md">
    <BackButton onClick={() => setBarState("origin")} />
    <ParseUrl />
  </div>
)

function TopBar() {
  const [barState, setBarState] = useState("origin")

  return (
    <div className="h-16">
      {
        (() => {
          switch (barState) {
            case "search":
              return <SearchTopBar setBarState={setBarState} />
            case "parse":
              return <ParseTopBar setBarState={setBarState} />
            default:
              return <OriginBar setBarState={setBarState} />
          }
        })()
      }
    </div>
  )
}

export default function MainPage() {
  return (
    <MainLayout top={<TopBar />} mainContent={<MainPageContent />} bottom={<MiniPlayer />} />
  )
}
