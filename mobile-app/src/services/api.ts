import axios from 'axios';
import AsyncStorage from '@react-native-async-storage/async-storage';

const API_BASE = __DEV__ ? 'http://192.168.1.100:8080/api' : 'https://api.promaxos.com/api';

const api = axios.create({
  baseURL: API_BASE,
  timeout: 15000,
  headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
});

api.interceptors.request.use(async (config) => {
  const token = await AsyncStorage.getItem('promax_token');
  if (token) config.headers.Authorization = `Bearer ${token}`;
  return config;
});

api.interceptors.response.use(
  (res) => res,
  (err) => {
    if (err.response?.status === 401) {
      AsyncStorage.removeItem('promax_token');
    }
    return Promise.reject(err);
  }
);

export interface LoginRequest { username: string; password: string; }
export interface LoginResponse { token: string; user: string; role: string; }
export interface DashboardData {
  customers: number; invoices: number; products: number; employees: number;
  revenue_omr: number; unpaid_invoices: number; low_stock_items: number; pending_shipments: number;
  recent_invoices: { id: number; invoice_no: string; date: string; customer_name: string; total_omr: number; status: string; }[];
}
export interface Customer { id: number; name: string; phone?: string; email?: string; vat_number?: string; credit_limit_omr: number; balance_omr: number; active: boolean; created_at: string; }
export interface Invoice { id: number; invoice_no: string; date: string; customer_name: string; customer_vat: string; net_omr: number; vat_omr: number; total_omr: number; status: string; notes?: string; lines: { product: string; qty: number; unit_price_omr: number; total_omr: number; }[]; }
export interface Product { id: number; code?: string; name_ar: string; name_en?: string; category?: string; unit_price_omr: number; active: boolean; }

export const auth = { login: (d: LoginRequest) => api.post<LoginResponse>('/auth/login', d) };
export const dashboard = { get: () => api.get<DashboardData>('/dashboard') };
export const customers = { list: () => api.get<Customer[]>('/customers'), get: (id: number) => api.get<Customer>(`/customers/${id}`) };
export const invoices = { list: (params?: any) => api.get<DashboardData['recent_invoices']>('/invoices', { params }), get: (id: number) => api.get<Invoice>(`/invoices/${id}`) };
export const products = { list: () => api.get<Product[]>('/products') };

export default api;
