import { useState } from "react"
import MiniPlayer from "../components/MiniPlayer"
import MainLayout from "../layout/MainLayout"
import { ChevronLeft, Link, Search } from "lucide-react"
import SearchInput from "../components/SearchBar"
import ParseUrl from "../components/ParseUrl"
import MainPageContent from "../components/MainPageContent"

const OriginBar = ({ setBarState }: { setBarState: Function }) => (<div className="flex w-full h-full px-6 flex-1 justify-between bg-gray-50/80 border-b border-gray-100 backdrop-blur-md">
  <button className="group transition-all duration-200 hover:scale-105 h-full" onClick={() => setBarState("search")}><Search className="bg-transparent text-gray-500  group-hover:text-purple-900 transition-colors" /></button>
  <span className="text-sm font-semibold tracking-widest uppercase">Menu</span>
  <button className="group transition-all duration-200 hover:scale-105 h-full" onClick={() => setBarState("parse")}><Link className="bg-transparent text-gray-500 group-hover:text-purple-900 transition-colors" /></button>
</div>)

const SearchTopBar = ({ setBarState }: { setBarState: Function }) => (
  <div className="flex flex-1 justify-between">
    <button onClick={() => { setBarState("origin") }}><ChevronLeft /></button>
    <SearchInput />
  </div>
)

const ParseTopBar = ({ setBarState }: { setBarState: Function }) => (
  <div className="flex flex-1 justify-between">
    <button onClick={() => { setBarState("origin") }}><ChevronLeft /></button>
    <ParseUrl />
  </div>
)



function TopBar() {
  const [barState, setBarState] = useState("origin")

  return (
    <div>
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
    <div>
      <MainLayout top={<TopBar />} mainContent={<MainPageContent />} bottom={<MiniPlayer />} />
    </div>
  )
}
