import { useNavigate, useLocation } from 'react-router-dom'
import { useState } from 'react'
import { useAuthStore } from '../../entities/session'
import { authApi } from '../../shared/api/authApi'
import styles from './LoginPage.module.css'

interface LoginCredentials {
  token: string
  memberId: string
  tenantId?: string
}

export function LoginPage() {
  const navigate = useNavigate()
  const location = useLocation()
  const { login } = useAuthStore()
  const [credentials, setCredentials] = useState<LoginCredentials>({
    token: '',
    memberId: '',
    tenantId: '',
  })
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  const from = (location.state as { from?: { pathname: string } })?.from?.pathname || '/app/rooms'

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (!credentials.token.trim()) {
      setError('Token is required')
      return
    }

    setLoading(true)
    try {
      login(await authApi.authenticate(credentials.token.trim()))
      navigate(from, { replace: true })
    } catch (err) {
      setError('Login failed. Please check your credentials.')
    } finally {
      setLoading(false)
    }
  }

  const handleChange = (field: keyof LoginCredentials) => (
    e: React.ChangeEvent<HTMLInputElement>
  ) => {
    setCredentials((prev) => ({ ...prev, [field]: e.target.value }))
  }

  return (
    <div className={styles.container}>
      <div className={styles.card}>
        <h1 className={styles.title}>Sign in to Wisdoverse Nexus</h1>
        <p className={styles.subtitle}>Enter your credentials to continue</p>

        {error && <div className={styles.error}>{error}</div>}

        <form onSubmit={handleSubmit} className={styles.form}>
          <div className={styles.field}>
            <label htmlFor="token">Token</label>
            <input
              id="token"
              type="password"
              value={credentials.token}
              onChange={handleChange('token')}
              placeholder="Enter your API token"
              autoComplete="off"
            />
          </div>

          <button type="submit" disabled={loading} className={styles.submit}>
            {loading ? 'Signing in...' : 'Sign in'}
          </button>
        </form>
      </div>
    </div>
  )
}

export default LoginPage
