import React, { useState } from 'react';
import { View, Text, TextInput, TouchableOpacity, StyleSheet, ActivityIndicator, KeyboardAvoidingView, Platform } from 'react-native';
import { useAuth } from '../context/AuthContext';
import { useTranslation } from 'react-i18next';

const LoginScreen: React.FC = () => {
  const { t } = useTranslation();
  const { login } = useAuth();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleLogin = async () => {
    if (!username.trim() || !password.trim()) { setError('Please enter credentials'); return; }
    setLoading(true); setError('');
    try {
      await login({ username: username.trim(), password });
    } catch (e: any) {
      setError(e?.response?.data?.error || t('error'));
    } finally { setLoading(false); }
  };

  return (
    <KeyboardAvoidingView style={styles.container} behavior={Platform.OS === 'ios' ? 'padding' : undefined}>
      <View style={styles.card}>
        <Text style={styles.logo}>PRO MAX OS</Text>
        <Text style={styles.subtitle}>{t('login')}</Text>

        <TextInput style={styles.input} placeholder={t('username')} placeholderTextColor="#888"
          value={username} onChangeText={setUsername} autoCapitalize="none" autoCorrect={false} />
        <TextInput style={styles.input} placeholder={t('password')} placeholderTextColor="#888"
          value={password} onChangeText={setPassword} secureTextEntry />

        {error ? <Text style={styles.error}>{error}</Text> : null}

        <TouchableOpacity style={styles.button} onPress={handleLogin} disabled={loading}>
          {loading ? <ActivityIndicator color="#fff" /> : <Text style={styles.buttonText}>{t('login')}</Text>}
        </TouchableOpacity>
      </View>
    </KeyboardAvoidingView>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, justifyContent: 'center', alignItems: 'center', backgroundColor: '#0D1117' },
  card: { width: '85%', maxWidth: 400, backgroundColor: '#161B22', borderRadius: 16, padding: 32, borderWidth: 1, borderColor: '#30363D' },
  logo: { fontSize: 32, fontWeight: '800', color: '#F0B429', textAlign: 'center', marginBottom: 8 },
  subtitle: { fontSize: 16, color: '#8B949E', textAlign: 'center', marginBottom: 32 },
  input: { backgroundColor: '#0D1117', borderRadius: 10, padding: 14, fontSize: 16, color: '#F0F0F0', marginBottom: 16, borderWidth: 1, borderColor: '#30363D' },
  button: { backgroundColor: '#F0B429', borderRadius: 10, padding: 16, alignItems: 'center', marginTop: 8 },
  buttonText: { color: '#0D1117', fontSize: 16, fontWeight: '700' },
  error: { color: '#F85149', textAlign: 'center', marginBottom: 12, fontSize: 14 },
});

export default LoginScreen;
