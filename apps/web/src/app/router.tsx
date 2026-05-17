import { createBrowserRouter, Navigate } from 'react-router-dom'
import { ProtectedRoute } from './routes/ProtectedRoute'
import { AppShell } from '../widgets/app-shell'
import LoginPage from '../pages/Login'
import MembersPage from '../pages/MembersPage'
import RoomPage from '../pages/RoomPage'
import RoomsPage from '../pages/Home'
import SearchPage from '../pages/SearchPage'

export const router = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },
  {
    path: '/app',
    element: (
      <ProtectedRoute>
        <AppShell />
      </ProtectedRoute>
    ),
    children: [
      {
        index: true,
        element: <Navigate to="/app/rooms" replace />,
      },
      {
        path: 'rooms',
        element: <RoomsPage />,
      },
      {
        path: 'rooms/:roomId',
        element: <RoomPage />,
      },
      {
        path: 'members',
        element: <MembersPage />,
      },
      {
        path: 'search',
        element: <SearchPage />,
      },
    ],
  },
  {
    path: '/',
    element: <Navigate to="/app/rooms" replace />,
  },
])
