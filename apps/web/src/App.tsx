import { RouterProvider } from 'react-router-dom'
import { Providers, router } from './app'

export default function App() {
  return (
    <Providers>
      <RouterProvider router={router} />
    </Providers>
  )
}
