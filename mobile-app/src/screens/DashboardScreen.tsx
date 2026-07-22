import React, { useEffect, useState, useCallback } from 'react';
import { View, Text, ScrollView, TouchableOpacity, RefreshControl, StyleSheet, ActivityIndicator } from 'react-native';
import { useTranslation } from 'react-i18next';
import { dashboard, DashboardData } from '../services/api';
import { useAuth } from '../context/AuthContext';

const StatCard: React.FC<{ label: string; value: string | number; color: string }> = ({ label, value, color }) => (
  <View style={[styles.statCard, { borderLeftColor: color }]}>
    <Text style={[styles.statValue, { color }]}>{typeof value === 'number' ? value.toLocaleString() : value}</Text>
    <Text style={styles.statLabel}>{label}</Text>
  </View>
);

const DashboardScreen: React.FC = () => {
  const { t } = useTranslation();
  const { user, logout } = useAuth();
  const [data, setData] = useState<DashboardData | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  const load = useCallback(async () => {
    try { const res = await dashboard.get(); setData(res.data); } catch {}
  }, []);

  useEffect(() => { load(); }, [load]);

  const onRefresh = async () => { setRefreshing(true); await load(); setRefreshing(false); };

  return (
    <ScrollView style={styles.container} refreshControl={<RefreshControl refreshing={refreshing} onRefresh={onRefresh} tintColor="#F0B429" />}>
      <View style={styles.header}>
        <View><Text style={styles.title}>PRO MAX OS</Text><Text style={styles.greeting}>{t('welcome')}, {user}</Text></View>
        <TouchableOpacity onPress={logout} style={styles.logoutBtn}><Text style={styles.logoutText}>{t('logout')}</Text></TouchableOpacity>
      </View>

      {!data ? <ActivityIndicator size="large" color="#F0B429" style={{ marginTop: 60 }} />
      : <><View style={styles.statsGrid}>
        <StatCard label={t('total_revenue')} value={`${data.revenue_omr.toFixed(3)} OMR`} color="#F0B429" />
        <StatCard label={t('invoices')} value={data.invoices} color="#58A6FF" />
        <StatCard label={t('customers')} value={data.customers} color="#3FB950" />
        <StatCard label={t('products')} value={data.products} color="#D2A8FF" />
        <StatCard label={t('employees')} value={data.employees} color="#F0883E" />
        <StatCard label={t('unpaid')} value={data.unpaid_invoices} color="#F85149" />
        <StatCard label={t('low_stock')} value={data.low_stock_items} color="#F85149" />
      </View>

      <Text style={styles.sectionTitle}>{t('recent_invoices')}</Text>
      {data.recent_invoices.map((inv) => (
        <View key={inv.id} style={styles.invoiceRow}>
          <View style={{ flex: 1 }}>
            <Text style={styles.invNo}>{inv.invoice_no}</Text>
            <Text style={styles.invCustomer}>{inv.customer_name}</Text>
            <Text style={styles.invDate}>{inv.date}</Text>
          </View>
          <View style={{ alignItems: 'flex-end' }}>
            <Text style={styles.invAmount}>{inv.total_omr.toFixed(3)} OMR</Text>
            <Text style={[styles.invStatus, { color: inv.status === 'paid' ? '#3FB950' : inv.status === 'cancelled' ? '#F85149' : '#F0B429' }]}>{inv.status}</Text>
          </View>
        </View>
      ))}
      {data.recent_invoices.length === 0 && <Text style={styles.empty}>{t('no_data')}</Text>}
      </>}
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0D1117' },
  header: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center', padding: 20, paddingTop: 50 },
  title: { fontSize: 22, fontWeight: '800', color: '#F0B429' },
  greeting: { fontSize: 14, color: '#8B949E', marginTop: 4 },
  logoutBtn: { paddingHorizontal: 16, paddingVertical: 8, borderRadius: 8, borderWidth: 1, borderColor: '#30363D' },
  logoutText: { color: '#F85149', fontSize: 14 },
  statsGrid: { flexDirection: 'row', flexWrap: 'wrap', padding: 10 },
  statCard: { width: '46%', margin: '2%', backgroundColor: '#161B22', borderRadius: 12, padding: 16, borderLeftWidth: 3, borderWidth: 1, borderColor: '#30363D' },
  statValue: { fontSize: 24, fontWeight: '800' },
  statLabel: { fontSize: 13, color: '#8B949E', marginTop: 4 },
  sectionTitle: { fontSize: 18, fontWeight: '700', color: '#F0F0F0', paddingHorizontal: 20, marginTop: 10, marginBottom: 8 },
  invoiceRow: { flexDirection: 'row', backgroundColor: '#161B22', marginHorizontal: 20, marginVertical: 4, padding: 14, borderRadius: 10, borderWidth: 1, borderColor: '#30363D' },
  invNo: { fontSize: 15, fontWeight: '600', color: '#F0F0F0' },
  invCustomer: { fontSize: 13, color: '#8B949E', marginTop: 2 },
  invDate: { fontSize: 12, color: '#6E7681', marginTop: 2 },
  invAmount: { fontSize: 16, fontWeight: '700', color: '#F0B429' },
  invStatus: { fontSize: 12, fontWeight: '600', marginTop: 4 },
  empty: { color: '#6E7681', textAlign: 'center', marginTop: 20, fontSize: 14 },
});

export default DashboardScreen;
