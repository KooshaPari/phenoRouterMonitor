import React from 'react';
import ReactDOM from 'react-dom/client';

// Import global styles
import './styles/globals.css';

/**
 * Component Library Storybook
 * This is a development entry point for testing components
 * In production, components will be exported as a library
 */

function App() {
  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white border-b border-gray-200 px-6 py-4">
        <h1 className="text-2xl font-bold text-gray-900">
          AgilePlus Dashboard Components
        </h1>
        <p className="text-gray-600 mt-1">
          11 Foundation & Layout Components — Week 1 Complete
        </p>
      </header>

      <main className="p-6 max-w-7xl mx-auto">
        <section className="mb-12">
          <h2 className="text-xl font-semibold text-gray-900 mb-4">
            Foundation Components
          </h2>
          <div className="bg-white rounded-lg border border-gray-200 p-6">
            <p className="text-gray-600">
              6 components implemented: Button, Input, Select, Checkbox, Radio, Toggle
            </p>
            <p className="text-gray-500 text-sm mt-2">
              See src/components/foundation/ for implementations
            </p>
          </div>
        </section>

        <section className="mb-12">
          <h2 className="text-xl font-semibold text-gray-900 mb-4">
            Layout Components
          </h2>
          <div className="bg-white rounded-lg border border-gray-200 p-6">
            <p className="text-gray-600">
              5 components implemented: Card, Modal, Toast, Badge, Pill
            </p>
            <p className="text-gray-500 text-sm mt-2">
              See src/components/layout/ for implementations
            </p>
          </div>
        </section>

        <section>
          <h2 className="text-xl font-semibold text-gray-900 mb-4">
            Getting Started
          </h2>
          <div className="bg-white rounded-lg border border-gray-200 p-6 space-y-2">
            <p className="text-gray-600">
              <strong>Import components:</strong>
            </p>
            <pre className="bg-gray-50 p-3 rounded text-sm overflow-x-auto">
{`import { Button, Input, Card } from '@/components';`}
            </pre>

            <p className="text-gray-600 mt-4">
              <strong>Run tests:</strong>
            </p>
            <pre className="bg-gray-50 p-3 rounded text-sm overflow-x-auto">
{`npm run test              # Run all tests
npm run test:ui          # Vitest UI dashboard
npm run test:coverage    # Generate coverage report`}
            </pre>

            <p className="text-gray-600 mt-4">
              <strong>Documentation:</strong>
            </p>
            <ul className="list-disc list-inside text-gray-600 space-y-1">
              <li>WEEK1_COMPLETION.md — Full completion report</li>
              <li>COMPONENTS_QUICK_START.md — Usage guide with examples</li>
              <li>src/types/index.ts — TypeScript interfaces</li>
            </ul>
          </div>
        </section>
      </main>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
