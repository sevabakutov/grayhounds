import MainLayout from "@/layouts/MainLayout";
import { PredictionPage, SettingsPage, TestingPage } from "@/pages";
import { HashRouter, Route, Routes } from "react-router";

const Router = () => (
  <HashRouter>
    <Routes>
      <Route path="/" element={<MainLayout />}>
        <Route path="predict"  element={<PredictionPage />} />
        <Route path="test"     element={<TestingPage />} />
        <Route path="settings" element={<SettingsPage />} />
      </Route>
    </Routes>
  </HashRouter>
)

export default Router;