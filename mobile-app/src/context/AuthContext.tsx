import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { auth, LoginRequest, LoginResponse } from '../services/api';

interface AuthState {
  user: string | null;
  role: string | null;
  token: string | null;
  loading: boolean;
  login: (data: LoginRequest) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthState>({
  user: null, role: null, token: null, loading: true,
  login: async () => {}, logout: async () => {},
});

export const AuthProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [user, setUser] = useState<string | null>(null);
  const [role, setRole] = useState<string | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => { (async () => {
    const t = await AsyncStorage.getItem('promax_token');
    const u = await AsyncStorage.getItem('promax_user');
    const r = await AsyncStorage.getItem('promax_role');
    if (t && u) { setToken(t); setUser(u); setRole(r); }
    setLoading(false);
  })(); }, []);

  const login = async (data: LoginRequest) => {
    const res = await auth.login(data);
    await AsyncStorage.setItem('promax_token', res.data.token);
    await AsyncStorage.setItem('promax_user', res.data.user);
    await AsyncStorage.setItem('promax_role', res.data.role);
    setToken(res.data.token); setUser(res.data.user); setRole(res.data.role);
  };

  const logout = async () => {
    await AsyncStorage.multiRemove(['promax_token', 'promax_user', 'promax_role']);
    setToken(null); setUser(null); setRole(null);
  };

  return (
    <AuthContext.Provider value={{ user, role, token, loading, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => useContext(AuthContext);
