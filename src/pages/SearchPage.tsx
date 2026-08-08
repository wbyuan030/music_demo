import { ChevronLeft } from "lucide-react"
import SearchInput from "../components/SearchBar"
import { useStateStore } from "../store/State"
import MiniPlayer from "../components/MiniPlayer"
import MainLayout from "../layout/MainLayout"
import { StateEnum } from "../types/state"
import SearchContent from "../components/SearchContent"





function TopBar() {
  const setCurrentState = useStateStore((state) => state.setCurrentState)
  return (
    <div className="flex items-center gap-2 h-16 px-4 bg-neutral-900/70 border-b border-neutral-800 backdrop-blur-md">
      <button
        onClick={() => { setCurrentState(StateEnum.mainPage) }}
        className="w-10 h-10 flex items-center justify-center rounded-full text-neutral-400 hover:text-white hover:bg-neutral-800/80 transition-colors"
      ><ChevronLeft /></button>
      <SearchInput />
    </div>
  )
}

export default function SearchPage() {
  return (
    <div>
      <MainLayout top={<TopBar />} mainContent={< SearchContent />} bottom={<MiniPlayer />} />
    </div>
  )
}
