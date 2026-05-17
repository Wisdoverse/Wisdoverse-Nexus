import { create } from 'zustand'
import { useAuthStore } from '../../entities/session'

export interface Tenant {
  id: string
  name: string
}

interface TenantState {
  tenantId: string | null
  availableTenants: Tenant[]
  setTenantId: (id: string) => void
  setAvailableTenants: (tenants: Tenant[]) => void
}

export const useTenantStore = create<TenantState>((set) => ({
  tenantId: null,
  availableTenants: [],
  setTenantId: (id: string) => {
    set({ tenantId: id })
    useAuthStore.getState().setTenantId(id)
  },
  setAvailableTenants: (tenants: Tenant[]) => set({ availableTenants: tenants }),
}))
