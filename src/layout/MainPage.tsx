// import { useState } from 'react'
// import 'App.css'
import type { ReactNode } from "react"

interface MainpageProps {
  left: ReactNode;
  right: ReactNode;
  top: ReactNode;
  bottom: ReactNode;
}

function Mainpage({ left, right, top, bottom }: MainpageProps) {

  return (
    <>
      <div className="flex flex-col h-screen  ">
        <div className="h-2/12 mt-2">
          {top}
        </div>
        <div className="flex flex-1 flex-row gap-4 min-h-0">
          <div className="w-3/10  bg-neutral-50 rounded-xl">
            {left}
          </div>
          <div className="w-7/10 bg-neutral-900 rounded-xl overflow-y-auto p-4 rounded">
            {right}
          </div>
        </div>
        <div className="h-1/12">
          {bottom}
        </div>
      </div>
    </>
  )
}

export default Mainpage
