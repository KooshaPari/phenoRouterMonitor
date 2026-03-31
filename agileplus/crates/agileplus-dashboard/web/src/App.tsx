import { useState } from 'react'
import { Dashboard } from './pages/Dashboard'
import { Settings } from './pages/Settings'
import './App.css'

type Page = 'dashboard' | 'settings'

function App() {
  const [currentPage, setCurrentPage] = useState<Page>('dashboard')

  return (
    <div className="min-h-screen bg-background">
      {/* Navigation */}
      <nav className="border-b border-border bg-card shadow-sm sticky top-0 z-40">
        <div className="max-w-7xl mx-auto px-8 py-4 flex items-center justify-between">
          <div className="flex items-center gap-8">
            <h1 className="text-xl font-bold text-primary">AgilePlus Dashboard</h1>
            <div className="flex gap-1">
              <button
                onClick={() => setCurrentPage('dashboard')}
                className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
                  currentPage === 'dashboard'
                    ? 'bg-primary text-primary-foreground'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                Dashboard
              </button>
              <button
                onClick={() => setCurrentPage('settings')}
                className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
                  currentPage === 'settings'
                    ? 'bg-primary text-primary-foreground'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                Settings
              </button>
            </div>
          </div>
          <div className="flex items-center gap-4">
            <span className="text-xs text-muted-foreground">
              v1.0.0 · React + shadcn/ui
            </span>
          </div>
        </div>
      </nav>

      {/* Page Content */}
      <main>
        {currentPage === 'dashboard' && <Dashboard />}
        {currentPage === 'settings' && <Settings />}
      </main>
    </div>
  )
}

export default App
