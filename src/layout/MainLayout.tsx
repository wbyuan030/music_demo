import { type ReactNode } from "react"

interface MainpageProps {
  mainContent: ReactNode;
  top: ReactNode;
  bottom: ReactNode;
}

function MainLayout({ mainContent, top, bottom }: MainpageProps) {
  return (
    <div className="flex flex-col h-screen w-screen bg-neutral-950">
      <header className="shrink-0">{top}</header>
      <main className="flex-1 min-h-0 overflow-y-auto pb-32 md:pb-0">{mainContent}</main>
      <footer className="shrink-0">{bottom}</footer>
    </div>
  )
}

export default MainLayout
