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
    <div className="flex flex-row flex-1 justify-between ">
      <button onClick={() => { setCurrentState(StateEnum.mainPage) }}><ChevronLeft /></button>
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
