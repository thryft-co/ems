import React, { useState, useEffect } from "react";
import { AuthProvider, useAuth } from "./contexts/AuthContext";
import { Sidebar } from "./components/sidebar";
import { Card, CardHeader, CardTitle, CardContent } from "./components/ui/card";
import { Button } from "./components/ui/button";
import AuthWrapper from "./components/auth/AuthWrapper";
import TenantSelection from "./components/auth/TenantSelection";
import {
  LayoutDashboard,
  ShoppingCart,
  Store,
  Factory,
  ClipboardCheck,
  Package,
  Truck,
  Users,
  User,
  Settings,
  LogOut,
  Building2,
} from "lucide-react";

// Import job components
import ManufacturingJobList from "./components/jobs/ManufacturingJobList";
import QAJobList from "./components/jobs/QAJobList";
import ServiceJobList from "./components/jobs/ServiceJobList";

// Import order components
import CustomerOrderList from "./components/orders/CustomerOrderList";
import PurchaseOrderList from "./components/orders/PurchaseOrderList";
import DistributorOrderList from "./components/orders/DistributorOrderList";

// Import person components
import InternalUserList from "./components/persons/InternalUserList";
import CustomerList from "./components/persons/CustomerList";
import VendorList from "./components/persons/VendorList";
import DistributorList from "./components/persons/DistributorList";

// Dashboard Component
const Dashboard: React.FC = () => {
  const { user, currentTenant, logout, isLoading } = useAuth();
  const [activeSection, setActiveSection] = useState("dashboard");
  const [activeTab, setActiveTab] = useState("");

  // Store tenant ID in localStorage for API calls
  useEffect(() => {
    if (currentTenant) {
      localStorage.setItem("tenantId", currentTenant.id);
    }
  }, [currentTenant]);

  // Handle logout
  const handleLogout = async () => {
    try {
      await logout();
    } catch (error) {
      console.error("Logout error:", error);
    }
  };

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  // Define the sidebar item type
  interface SidebarItemType {
    id: string;
    label: string;
    icon: React.ComponentType<any>;
    hasTabs?: boolean;
  }

  const sidebarItems: SidebarItemType[] = [
    { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
    { id: "purchase", label: "Purchase", icon: ShoppingCart, hasTabs: true },
    { id: "store", label: "Store", icon: Store },
    {
      id: "manufacturing",
      label: "Manufacturing",
      icon: Factory,
      hasTabs: true,
    },
    {
      id: "quality",
      label: "Quality & Compliance",
      icon: ClipboardCheck,
      hasTabs: true,
    },
    {
      id: "finishedgoods",
      label: "Finished Goods",
      icon: Package,
      hasTabs: true,
    },
    { id: "distribution", label: "Distribution", icon: Truck, hasTabs: true },
    { id: "customers", label: "Customers", icon: Users, hasTabs: true },
    { id: "users", label: "Users", icon: User, hasTabs: true },
    { id: "settings", label: "Settings", icon: Settings, hasTabs: true },
  ];

  // Define the tab item type
  interface TabItem {
    id: string;
    label: string;
  }

  // Define the section tabs type
  type SectionTabs = {
    [key: string]: TabItem[];
  };

  const sectionTabs: SectionTabs = {
    purchase: [
      { id: "orders", label: "Orders" },
      { id: "vendors", label: "Vendors" },
    ],
    manufacturing: [
      { id: "jobs", label: "Jobs" },
      { id: "firmware", label: "Firmware" },
      { id: "bom", label: "BOM" },
      { id: "testjig", label: "Test Jig Configuration" },
    ],
    quality: [
      { id: "jobs", label: "Jobs" },
      { id: "compliance", label: "Compliance" },
    ],
    finishedgoods: [
      { id: "inventory", label: "Inventory" },
      { id: "uniteconomics", label: "Unit Economics" },
    ],
    distribution: [
      { id: "orders", label: "Orders" },
      { id: "distributors", label: "Authorised Distributors" },
    ],
    customers: [
      { id: "list", label: "Customer List" },
      { id: "orders", label: "Customer Orders" },
      { id: "jobs", label: "Service Jobs" },
    ],
    users: [
      { id: "list", label: "Internal Users" },
      { id: "permissions", label: "Permissions" },
      { id: "roles", label: "Roles" },
    ],
    settings: [{ id: "alerts", label: "Alerts" }],
  };

  const handleSectionChange = (sectionId: string) => {
    setActiveSection(sectionId);
    // Set the default tab to the first available tab for this section
    const section = sectionTabs[sectionId as keyof typeof sectionTabs];
    if (section && section.length > 0) {
      setActiveTab(section[0].id);
    } else {
      // If no tabs available, use a default
      setActiveTab("");
    }
  };

  const handleTabChange = (tabId: string) => {
    setActiveTab(tabId);
  };

  const renderContent = () => {
    const currentItem = sidebarItems.find((item) => item.id === activeSection);

    if (!currentItem) return null;

    // Special handling for job components
    if (currentItem.hasTabs && activeTab === "jobs") {
      return (
        <Card className="w-full max-w-6xl mx-auto">
          <CardHeader>
            <CardTitle>{currentItem.label}</CardTitle>
            {sectionTabs[activeSection] && (
              <div className="flex space-x-2 mt-4 border-b">
                {sectionTabs[activeSection].map((tab) => (
                  <button
                    key={tab.id}
                    onClick={() => handleTabChange(tab.id)}
                    className={`px-4 py-2 text-sm font-medium transition-colors ${
                      activeTab === tab.id
                        ? "border-b-2 border-primary text-primary"
                        : "text-muted-foreground hover:text-foreground"
                    }`}
                  >
                    {tab.label}
                  </button>
                ))}
              </div>
            )}
          </CardHeader>
          <CardContent>
            {activeSection === "manufacturing" && <ManufacturingJobList />}
            {activeSection === "quality" && <QAJobList />}
            {activeSection === "customers" && <ServiceJobList />}
          </CardContent>
        </Card>
      );
    }

    // Special handling for order components
    if (currentItem.hasTabs && activeTab === "orders") {
      return (
        <Card className="w-full max-w-6xl mx-auto">
          <CardHeader>
            <CardTitle>{currentItem.label}</CardTitle>
            {sectionTabs[activeSection] && (
              <div className="flex space-x-2 mt-4 border-b">
                {sectionTabs[activeSection].map((tab) => (
                  <button
                    key={tab.id}
                    onClick={() => handleTabChange(tab.id)}
                    className={`px-4 py-2 text-sm font-medium transition-colors ${
                      activeTab === tab.id
                        ? "border-b-2 border-primary text-primary"
                        : "text-muted-foreground hover:text-foreground"
                    }`}
                  >
                    {tab.label}
                  </button>
                ))}
              </div>
            )}
          </CardHeader>
          <CardContent>
            {activeSection === "purchase" && <PurchaseOrderList />}
            {activeSection === "distribution" && <DistributorOrderList />}
            {activeSection === "customers" && <CustomerOrderList />}
          </CardContent>
        </Card>
      );
    }

    // Special handling for person list components
    if (
      currentItem.hasTabs &&
      (activeTab === "list" ||
        activeTab === "vendors" ||
        activeTab === "distributors")
    ) {
      return (
        <Card className="w-full max-w-6xl mx-auto">
          <CardHeader>
            <CardTitle>{currentItem.label}</CardTitle>
            {sectionTabs[activeSection] && (
              <div className="flex space-x-2 mt-4 border-b">
                {sectionTabs[activeSection].map((tab) => (
                  <button
                    key={tab.id}
                    onClick={() => handleTabChange(tab.id)}
                    className={`px-4 py-2 text-sm font-medium transition-colors ${
                      activeTab === tab.id
                        ? "border-b-2 border-primary text-primary"
                        : "text-muted-foreground hover:text-foreground"
                    }`}
                  >
                    {tab.label}
                  </button>
                ))}
              </div>
            )}
          </CardHeader>
          <CardContent>
            {activeSection === "users" && activeTab === "list" && (
              <InternalUserList />
            )}
            {activeSection === "customers" && activeTab === "list" && (
              <CustomerList />
            )}
            {activeSection === "purchase" && activeTab === "vendors" && (
              <VendorList />
            )}
            {activeSection === "distribution" &&
              activeTab === "distributors" && <DistributorList />}
          </CardContent>
        </Card>
      );
    }

    return (
      <Card className="w-full max-w-6xl mx-auto">
        <CardHeader>
          <CardTitle>{currentItem.label}</CardTitle>
          {currentItem.hasTabs && sectionTabs[activeSection] && (
            <div className="flex space-x-2 mt-4 border-b">
              {sectionTabs[activeSection].map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => handleTabChange(tab.id)}
                  className={`px-4 py-2 text-sm font-medium transition-colors ${
                    activeTab === tab.id
                      ? "border-b-2 border-primary text-primary"
                      : "text-muted-foreground hover:text-foreground"
                  }`}
                >
                  {tab.label}
                </button>
              ))}
            </div>
          )}
        </CardHeader>
        <CardContent>
          {currentItem.hasTabs && sectionTabs[activeSection] ? (
            <div>
              <h3 className="text-lg font-medium mb-2">
                {currentItem.label} -{" "}
                {
                  sectionTabs[activeSection].find((tab) => tab.id === activeTab)
                    ?.label
                }
              </h3>
              <p className="text-muted-foreground">
                This tab is currently empty.
              </p>
            </div>
          ) : (
            <p className="text-muted-foreground">
              This section is currently empty.
            </p>
          )}
        </CardContent>
      </Card>
    );
  };

  return (
    <div className="flex flex-col h-screen w-full bg-background">
      <header className="h-16 border-b border-border/50 flex items-center justify-between px-6 sticky top-0 z-10 bg-background/80 backdrop-blur-xl shadow-sm">
        <span className="text-lg font-semibold tracking-tight text-foreground">
          Enterprise Management Suite
        </span>
        <div className="flex items-center gap-4">
          <Button
            variant="outline"
            size="sm"
            className="relative flex items-center gap-2 px-2 py-1 rounded-lg bg-secondary/50 w-auto min-w-[140px]"
          >
            <div className="absolute left-2 top-1/2 transform -translate-y-1/2 w-1.5 h-1.5 rounded-full bg-success animate-pulse" />
            <div className="flex items-center gap-2 pl-3">
              <Building2 className="w-3.5 h-3.5 text-primary" />
              <span className="text-sm font-medium text-foreground/80 truncate max-w-[100px]">
                {currentTenant?.name}
              </span>
            </div>
          </Button>
          <Button
            variant="outline"
            size="sm"
            className="flex items-center gap-2 px-2 py-1 rounded-lg bg-secondary/50 w-auto min-w-[140px]"
          >
            <User className="w-3.5 h-3.5 text-primary" />
            <span className="text-sm text-muted-foreground truncate max-w-[100px]">
              {user?.first_name} {user?.last_name}
            </span>
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={handleLogout}
            className="border-destructive text-destructive w-32"
          >
            <LogOut className="w-4 h-4" />
            <span className="text-sm">Sign Out</span>
          </Button>
        </div>
      </header>
      <div className="flex flex-1 min-h-0">
        <div className="w-64 h-full flex-shrink-0">
          <Sidebar
            items={sidebarItems}
            activeItem={activeSection}
            onItemClick={handleSectionChange}
          />
        </div>
        <div className="flex flex-col flex-1 min-w-0">
          <main className="flex-1 p-6 overflow-auto bg-muted/30">
            <div className="animate-fade-in">{renderContent()}</div>
          </main>
        </div>
      </div>
    </div>
  );
};

// Main App Component with Authentication Flow
const App: React.FC = () => {
  return (
    <AuthProvider>
      <AppContent />
    </AuthProvider>
  );
};

// App Content Component that handles routing based on auth state
const AppContent: React.FC = () => {
  const { isAuthenticated, currentTenant, isLoading } = useAuth();

  // Loading state
  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="flex flex-col items-center gap-4">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-primary to-primary/70 flex items-center justify-center shadow-soft animate-pulse">
            <span className="text-white font-bold text-lg">E</span>
          </div>
          <div className="text-sm text-muted-foreground">Loading...</div>
        </div>
      </div>
    );
  }

  // Not authenticated - show login/register
  if (!isAuthenticated) {
    return <AuthWrapper />;
  }

  // Authenticated but no tenant selected - show tenant selection
  if (isAuthenticated && !currentTenant) {
    return <TenantSelection />;
  }

  // Authenticated and tenant selected - show dashboard
  return <Dashboard />;
};

export default App;
