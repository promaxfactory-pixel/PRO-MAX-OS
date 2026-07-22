import React from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { StatusBar } from 'react-native';
import { AuthProvider, useAuth } from './context/AuthContext';
import LoginScreen from './screens/LoginScreen';
import DashboardScreen from './screens/DashboardScreen';
import CustomersScreen from './screens/CustomersScreen';
import InvoicesScreen from './screens/InvoicesScreen';
import ProductsScreen from './screens/ProductsScreen';
import './i18n';

type RootStackParamList = {
  Login: undefined;
  Main: undefined;
  Customers: undefined;
  Invoices: undefined;
  Products: undefined;
};

const Stack = createNativeStackNavigator<RootStackParamList>();

const MainTabs: React.FC = () => {
  const [activeTab, setActiveTab] = React.useState('Dashboard');

  const screens: Record<string, React.FC> = {
    Dashboard: DashboardScreen,
    Invoices: InvoicesScreen,
    Customers: CustomersScreen,
    Products: ProductsScreen,
  };

  const ScreenComp = screens[activeTab] || DashboardScreen;

  return (
    <>
      <ScreenComp />
      <TabBar active={activeTab} onSelect={setActiveTab} />
    </>
  );
};

const TabBar: React.FC<{ active: string; onSelect: (tab: string) => void }> = ({ active, onSelect }) => {
  const tabs = ['Dashboard', 'Invoices', 'Customers', 'Products'];
  const { Text, TouchableOpacity, View } = require('react-native');

  return (
    <View style={{ flexDirection: 'row', backgroundColor: '#161B22', borderTopWidth: 1, borderTopColor: '#30363D', paddingBottom: 25, paddingTop: 8 }}>
      {tabs.map((tab) => (
        <TouchableOpacity key={tab} style={{ flex: 1, alignItems: 'center' }} onPress={() => onSelect(tab)}>
          <Text style={{ color: active === tab ? '#F0B429' : '#6E7681', fontSize: 12, fontWeight: active === tab ? '700' : '500' }}>{tab}</Text>
        </TouchableOpacity>
      ))}
    </View>
  );
};

const Navigator: React.FC = () => {
  const { token, loading } = useAuth();
  if (loading) return null;
  return (
    <Stack.Navigator screenOptions={{ headerShown: false, contentStyle: { backgroundColor: '#0D1117' } }}>
      {!token ? <Stack.Screen name="Login" component={LoginScreen} />
      : <Stack.Screen name="Main" component={MainTabs} />}
    </Stack.Navigator>
  );
};

const App: React.FC = () => (
  <AuthProvider>
    <StatusBar barStyle="light-content" backgroundColor="#0D1117" />
    <NavigationContainer>
      <Navigator />
    </NavigationContainer>
  </AuthProvider>
);

export default App;
