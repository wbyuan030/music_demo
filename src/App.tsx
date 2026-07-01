import './App.css'
import { useStateStore } from './store/State'
import { StateEnum } from './types/state'
import TrackPage from './pages/TrackPage'
import MainPage from './pages/MainPage'
import SearchPage from './pages/SearchPage'
import Toast from './components/Toast'

function App() {
  const currentState = useStateStore((state) => state.currentState)

  let CurrentPage = () => {
    switch (currentState) {
      default:
        return <MainPage />
      case StateEnum.detail:
        return <TrackPage />
      case StateEnum.searchResult:
        return <SearchPage />
    }
  }
  return (
    <>
      <CurrentPage />
      <Toast />
    </>
  )
}

export default App
