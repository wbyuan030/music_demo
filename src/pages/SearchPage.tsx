import { ChevronLeft } from "lucide-react"
import SearchInput from "../components/SearchBar"
import { useStateStore } from "../store/State"
import MiniPlayer from "../components/player/MiniPlayer"
import MainLayout from "../layout/MainLayout"
import { StateEnum } from "../types/state"

const SearchTopBar = ({ setBarState }: { setBarState: Function }) => (
  <div className="flex flex-1 justify-between">
    <button onClick={() => { setBarState(StateEnum.mainPage) }}><ChevronLeft /></button>
    <SearchInput />
  </div>
)


function TopBar() {
  const setCurrentState = useStateStore((state) => state.setCurrentState)
  return (
    <div>
      <button onClick={() => { setCurrentState(StateEnum.mainPage) }}><ChevronLeft /></button>
      <SearchInput />
      {
        <SearchTopBar setBarState={setCurrentState}></SearchTopBar>
      }
    </div>
  )
}

export default function SearchPage() {
  return (
    <div>
      <MainLayout top={<TopBar />} mainContent={<SearchPage />} bottom={<MiniPlayer />} />
    </div>
  )
}
