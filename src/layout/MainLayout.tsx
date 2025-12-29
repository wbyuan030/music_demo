import { useState, type ReactNode } from "react"

interface MainpageProps {
  mainContent: ReactNode;
  top: ReactNode;
  bottom: ReactNode;
}

function MainLayout({ mainContent, top, bottom }: MainpageProps) {
  return (
    <>
      <div className="flex flex-col h-screen w-screen gap-2">
        <div className="h-3/12 mt-2 pb-2">
          {top}
        </div>
        <div className="flex flex-1 flex-row gap-4 ">
          {mainContent}
        </div>
        <div className="h-1/12">
          {bottom}
        </div>
      </div >
    </>
  )
}

export default MainLayout
