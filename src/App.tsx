import { useEffect, useState } from 'react'
// import reactLogo from './assets/react.svg'
// import viteLogo from '/vite.svg'
import './App.css'
import Mainpage from './layout/MainPage'
import GoogleSearchInput from './components/SearchBar'
import MiniPlayer from './components/player/MiniPlayer'
import { selectFile } from './Library'
import { Dialog } from './dialog'
import { useDialogStore } from './store/Dialog'
import SearchPage from './components/SearchPage'
import { useStateStore } from './store/State'
import { StateEnum } from './types/state'
import TrackPage from './components/TrackPage'


function Bottom() {
  return (
    <>
      <MiniPlayer></MiniPlayer>
    </>
  )
}

function Left() {
  return (
    <>
      <button onClick={selectFile}>
        选择文件
      </button>
      <button onClick={useDialogStore(state => state.handleOpen)}>
        弹窗
      </button>
    </>
  )
}

function Right() {
  const currentState = useStateStore((state) => state.currentState)
  const pageMap = {
    [StateEnum.detail]: <TrackPage />,
    [StateEnum.searchResult]: <SearchPage />
  }
  return (
    <>
      {pageMap[currentState] || null}
    </>
  )
}

function App() {
  return (
    <>
      <Dialog />
      <Mainpage
        left=<Left />
        right=<Right />
        top=<GoogleSearchInput />
        bottom=<Bottom />
      />
      {/* Mainpage(left(),right(),top(),down()) */}
    </>
  )
}

export default App
