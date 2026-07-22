import React, { useEffect, useState } from 'react';
import { View, Text, ScrollView, TextInput, StyleSheet, ActivityIndicator } from 'react-native';
import { useTranslation } from 'react-i18next';
import { products, Product } from '../services/api';

const ProductsScreen: React.FC = () => {
  const { t } = useTranslation();
  const [list, setList] = useState<Product[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');

  useEffect(() => { (async () => {
    try { const res = await products.list(); setList(res.data); } catch {} finally { setLoading(false); }
  })(); }, []);

  const filtered = list.filter((p) =>
    p.name_ar.includes(search) || (p.name_en || '').toLowerCase().includes(search.toLowerCase()) || (p.code || '').includes(search)
  );

  return (
    <View style={styles.container}>
      <Text style={styles.title}>{t('products')}</Text>
      <TextInput style={styles.search} placeholder={t('search')} placeholderTextColor="#6E7681"
        value={search} onChangeText={setSearch} />

      {loading ? <ActivityIndicator size="large" color="#F0B429" style={{ marginTop: 40 }} />
      : <ScrollView>{filtered.map((p) => (
        <View key={p.id} style={styles.card}>
          <Text style={styles.name}>{p.name_ar}</Text>
          {p.name_en ? <Text style={styles.en}>{p.name_en}</Text> : null}
          <View style={styles.badgeRow}>
            {p.code ? <Text style={styles.badge}>{p.code}</Text> : null}
            {p.category ? <Text style={styles.badge}>{p.category}</Text> : null}
            <Text style={[styles.badge, { color: '#F0B429' }]}>{p.unit_price_omr.toFixed(3)} OMR</Text>
          </View>
        </View>
      ))}</ScrollView>}
    </View>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0D1117', paddingTop: 50 },
  title: { fontSize: 22, fontWeight: '800', color: '#F0B429', paddingHorizontal: 20, marginBottom: 12 },
  search: { backgroundColor: '#161B22', marginHorizontal: 20, padding: 12, borderRadius: 10, fontSize: 15, color: '#F0F0F0', borderWidth: 1, borderColor: '#30363D', marginBottom: 12 },
  card: { backgroundColor: '#161B22', marginHorizontal: 20, marginVertical: 5, padding: 16, borderRadius: 12, borderWidth: 1, borderColor: '#30363D' },
  name: { fontSize: 17, fontWeight: '700', color: '#F0F0F0' },
  en: { fontSize: 14, color: '#8B949E', marginTop: 2 },
  badgeRow: { flexDirection: 'row', marginTop: 8, gap: 8, flexWrap: 'wrap' },
  badge: { fontSize: 12, color: '#8B949E', paddingHorizontal: 8, paddingVertical: 3, borderRadius: 6, backgroundColor: '#0D1117', overflow: 'hidden' },
});

export default ProductsScreen;
