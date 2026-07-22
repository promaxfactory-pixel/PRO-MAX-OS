import React, { useEffect, useState } from 'react';
import { View, Text, ScrollView, TouchableOpacity, StyleSheet, ActivityIndicator } from 'react-native';
import { useTranslation } from 'react-i18next';
import { invoices, DashboardData } from '../services/api';

type InvSummary = DashboardData['recent_invoices'][0];

const statusColors: Record<string, string> = { paid: '#3FB950', pending: '#F0B429', overdue: '#F85149', draft: '#8B949E', cancelled: '#F85149' };

const InvoicesScreen: React.FC = () => {
  const { t } = useTranslation();
  const [list, setList] = useState<InvSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<string | null>(null);

  useEffect(() => { load(); }, [filter]);
  const load = async () => {
    setLoading(true);
    try { const res = await invoices.list(filter ? { status: filter } : {}); setList(res.data); } catch {} finally { setLoading(false); }
  };

  const filters = ['all', 'paid', 'pending', 'draft', 'cancelled'];

  return (
    <View style={styles.container}>
      <Text style={styles.title}>{t('invoices')}</Text>

      <ScrollView horizontal showsHorizontalScrollIndicator={false} style={styles.filterRow}>
        {filters.map((f) => (
          <TouchableOpacity key={f} style={[styles.filterBtn, f === (filter || 'all') && styles.filterActive]}
            onPress={() => setFilter(f === 'all' ? null : f)}>
            <Text style={[styles.filterText, f === (filter || 'all') && styles.filterTextActive]}>{f}</Text>
          </TouchableOpacity>
        ))}
      </ScrollView>

      {loading ? <ActivityIndicator size="large" color="#F0B429" style={{ marginTop: 40 }} />
      : <ScrollView>{list.map((inv) => (
        <View key={inv.id} style={styles.card}>
          <View style={styles.cardTop}>
            <Text style={styles.invNo}>{inv.invoice_no}</Text>
            <Text style={[styles.status, { color: statusColors[inv.status] || '#F0B429' }]}>{inv.status}</Text>
          </View>
          <Text style={styles.customer}>{inv.customer_name}</Text>
          <View style={styles.cardBottom}>
            <Text style={styles.date}>{inv.date}</Text>
            <Text style={styles.amount}>{inv.total_omr.toFixed(3)} OMR</Text>
          </View>
        </View>
      ))}</ScrollView>}
    </View>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0D1117', paddingTop: 50 },
  title: { fontSize: 22, fontWeight: '800', color: '#F0B429', paddingHorizontal: 20, marginBottom: 12 },
  filterRow: { paddingHorizontal: 20, marginBottom: 12, maxHeight: 40 },
  filterBtn: { paddingHorizontal: 16, paddingVertical: 7, borderRadius: 20, backgroundColor: '#161B22', marginRight: 8, borderWidth: 1, borderColor: '#30363D' },
  filterActive: { backgroundColor: '#F0B429' },
  filterText: { color: '#8B949E', fontSize: 14, fontWeight: '600' },
  filterTextActive: { color: '#0D1117' },
  card: { backgroundColor: '#161B22', marginHorizontal: 20, marginVertical: 5, padding: 16, borderRadius: 12, borderWidth: 1, borderColor: '#30363D' },
  cardTop: { flexDirection: 'row', justifyContent: 'space-between', marginBottom: 4 },
  invNo: { fontSize: 15, fontWeight: '700', color: '#F0F0F0' },
  status: { fontSize: 12, fontWeight: '700', textTransform: 'uppercase' },
  customer: { fontSize: 14, color: '#8B949E', marginBottom: 8 },
  cardBottom: { flexDirection: 'row', justifyContent: 'space-between' },
  date: { fontSize: 12, color: '#6E7681' },
  amount: { fontSize: 16, fontWeight: '700', color: '#F0B429' },
});

export default InvoicesScreen;
