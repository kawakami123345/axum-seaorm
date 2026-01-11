import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import MainLayout from './layouts/MainLayout';
import Library from './pages/Library';
import Publishers from './pages/Publishers';
import Login from './pages/Login';

function App() {
  return (
    <Router>
      <Routes>
        <Route path="/login" element={<Login />} />
        <Route path="/" element={<MainLayout />}>
          <Route index element={<Navigate to="/library" replace />} />
          <Route path="dashboard" element={<div className="p-8 text-2xl font-bold">ダッシュボード (準備中)</div>} />
          <Route path="library" element={<Library />} />
          <Route path="publishers" element={<Publishers />} />
          <Route path="reports" element={<div className="p-8 text-2xl font-bold">レポート (準備中)</div>} />
        </Route>
      </Routes>
    </Router>

  );
}

export default App;
