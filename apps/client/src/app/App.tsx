import React, { useState, useEffect } from "react";
import AuthWrapper from "@/features/auth/components/AuthWrapper";
import { AuthProvider, useAuth } from "@/features/auth/context/AuthContext";
import ManufacturingJobList from "@/features/jobs/components/ManufacturingJobList";
import QAJobList from "@/features/jobs/components/QAJobList";
import ServiceJobList from "@/features/jobs/components/ServiceJobList";
import CustomerOrderList from "@/features/orders/components/CustomerOrderList";
import DistributorOrderList from "@/features/orders/components/DistributorOrderList";
import PurchaseOrderList from "@/features/orders/components/PurchaseOrderList";
import CustomerList from "@/features/persons/components/CustomerList";
import DistributorList from "@/features/persons/components/DistributorList";
import InternalUserList from "@/features/persons/components/InternalUserList";
import VendorList from "@/features/persons/components/VendorList";
import TenantSelection from "@/features/tenants/components/TenantSelection";
import { Sidebar } from "@/shared/components/Sidebar";
import { Button } from "@/shared/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
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
  MoreHorizontal,
  X,
  ChevronRight,
} from "lucide-react";


// Bottom Tab Bar for mobile navigation
interface BottomTabBarProps {
  items: { id: string; label: string; icon: React.ComponentType<any> }[];
  activeItem: string;
  onItemClick: (id: string) => void;
  overflowItems: { id: string; label: string; icon: React.ComponentType<any> }[];
}

const BottomTabBar: React.FC<BottomTabBarProps> = ({
  items,
  activeItem,
  onItemClick,
  overflowItems,
}) => {
  const [showMore, setShowMore] = useState(false);

  return (
    <>
      {/* More sheet overlay */}
      {showMore && (
        <div
          className="fixed inset-0 bg-black/30 z-40 md:hidden animate-fade-in"
          onClick={() => setShowMore(false)}
        />
      )}

      {/* More sheet */}
      {showMore && (
        <div className="fixed bottom-0 left-0 right-0 z-50 md:hidden animate-slide-up-sheet">
          <div className="bg-card rounded-t-3xl shadow-soft-xl border-t-[0.5px] border-border/50 safe-area-bottom">
            <div className="flex items-center justify-between px-5 pt-4 pb-2">
              <h3 className="text-[17px] font-semibold text-foreground">More</h3>
              <button
                onClick={() => setShowMore(false)}
                className="w-8 h-8 rounded-full bg-secondary/60 flex items-center justify-center"
              >
                <X className="w-4 h-4 text-muted-foreground" />
              </button>
            </div>
            <div className="px-2 pb-4 space-y-0.5">
              {overflowItems.map((item) => (
                <button
                  key={item.id}
                  onClick={() => {
                    onItemClick(item.id);
                    setShowMore(false);
                  }}
                  className={`flex items-center gap-3 w-full px-4 py-3 rounded-xl text-[15px] font-medium transition-all ${
                    activeItem === item.id
                      ? "bg-primary/10 text-primary"
                      : "text-foreground/70 hover:bg-secondary/50"
                  }`}
                >
                  <item.icon className={`w-5 h-5 ${activeItem === item.id ? "text-primary" : "text-foreground/40"}`} />
                  <span>{item.label}</span>
                  <ChevronRight className="w-4 h-4 ml-auto text-foreground/20" />
                </button>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Tab bar */}
      <div className="bottom-tab-bar md:hidden">
        <div className="flex items-center justify-around px-2 pt-2 pb-1">
          {items.map((item) => (
            <button
              key={item.id}
              onClick={() => onItemClick(item.id)}
              className="flex flex-col items-center gap-0.5 py-1 px-3 min-w-0 touch-target"
            >
              <item.icon
                className={`w-[22px] h-[22px] transition-colors ${
                  activeItem === item.id
                    ? "text-primary"
                    : "text-foreground/35"
                }`}
              />
              <span
                className={`text-[10px] font-medium truncate max-w-[64px] ${
                  activeItem === item.id
                    ? "text-primary"
                    : "text-foreground/35"
                }`}
              >
                {item.label}
              </span>
            </button>
          ))}
          {overflowItems.length > 0 && (
            <button
              onClick={() => setShowMore(true)}
              className="flex flex-col items-center gap-0.5 py-1 px-3 min-w-0 touch-target"
            >
              <MoreHorizontal
                className={`w-[22px] h-[22px] transition-colors ${
                  showMore || overflowItems.some(i => i.id === activeItem)
                    ? "text-primary"
                    : "text-foreground/35"
                }`}
              />
              <span
                className={`text-[10px] font-medium ${
                  showMore || overflowItems.some(i => i.id === activeItem)
                    ? "text-primary"
                    : "text-foreground/35"
                }`}
              >
                More
              </span>
            </button>
          )}
        </div>
      </div>
    </>
  );
};


// Segmented Control component
interface SegmentedControlProps {
  tabs: { id: string; label: string }[];
  activeTab: string;
  onTabChange: (tabId: string) => void;
}

const SegmentedControl: React.FC<SegmentedControlProps> = ({
  tabs,
  activeTab,
  onTabChange,
}) => {
  return (
    <div className="segmented-control w-full sm:w-auto overflow-x-auto">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          onClick={() => onTabChange(tab.id)}
          className={`segmented-control-item ${
            activeTab === tab.id ? "active" : ""
          }`}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
};


// Dashboard Component
const Dashboard: React.FC = () => {
  const { user, currentTenant, logout, selectTenant, isLoading } = useAuth();
  const [activeSection, setActiveSection] = useState("dashboard");
  const [activeTab, setActiveTab] = useState("");
  const [showMobileUserMenu, setShowMobileUserMenu] = useState(false);

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
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="flex flex-col items-center gap-3">
          <div className="w-10 h-10 rounded-2xl bg-primary flex items-center justify-center animate-pulse">
            <span className="text-white font-bold text-lg">E</span>
          </div>
          <div className="text-[13px] text-muted-foreground">Loading...</div>
        </div>
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
      label: "Quality",
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

  // Mobile tab bar: first 4 items + "More" for the rest
  const mobileTabItems = sidebarItems.slice(0, 4);
  const mobileOverflowItems = sidebarItems.slice(4);

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
      { id: "testjig", label: "Test Jig" },
    ],
    quality: [
      { id: "jobs", label: "Jobs" },
      { id: "compliance", label: "Compliance" },
    ],
    finishedgoods: [
      { id: "inventory", label: "Inventory" },
      { id: "uniteconomics", label: "Economics" },
    ],
    distribution: [
      { id: "orders", label: "Orders" },
      { id: "distributors", label: "Distributors" },
    ],
    customers: [
      { id: "list", label: "Customers" },
      { id: "orders", label: "Orders" },
      { id: "jobs", label: "Service" },
    ],
    users: [
      { id: "list", label: "Users" },
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

    // Build the inner content based on active section + tab
    let innerContent: React.ReactNode = null;

    // Job components
    if (currentItem.hasTabs && activeTab === "jobs") {
      if (activeSection === "manufacturing") innerContent = <ManufacturingJobList />;
      else if (activeSection === "quality") innerContent = <QAJobList />;
      else if (activeSection === "customers") innerContent = <ServiceJobList />;
    }

    // Order components
    if (currentItem.hasTabs && activeTab === "orders") {
      if (activeSection === "purchase") innerContent = <PurchaseOrderList />;
      else if (activeSection === "distribution") innerContent = <DistributorOrderList />;
      else if (activeSection === "customers") innerContent = <CustomerOrderList />;
    }

    // Person list components
    if (currentItem.hasTabs && (activeTab === "list" || activeTab === "vendors" || activeTab === "distributors")) {
      if (activeSection === "users" && activeTab === "list") innerContent = <InternalUserList />;
      else if (activeSection === "customers" && activeTab === "list") innerContent = <CustomerList />;
      else if (activeSection === "purchase" && activeTab === "vendors") innerContent = <VendorList />;
      else if (activeSection === "distribution" && activeTab === "distributors") innerContent = <DistributorList />;
    }

    // Fallback placeholder
    if (!innerContent) {
      if (currentItem.hasTabs && sectionTabs[activeSection]) {
        const tabLabel = sectionTabs[activeSection].find((tab) => tab.id === activeTab)?.label;
        innerContent = (
          <div className="flex flex-col items-center justify-center py-16 text-center">
            <div className="w-12 h-12 rounded-2xl bg-secondary/60 flex items-center justify-center mb-4">
              <currentItem.icon className="w-6 h-6 text-muted-foreground/50" />
            </div>
            <h3 className="text-[17px] font-semibold text-foreground mb-1">
              {tabLabel || currentItem.label}
            </h3>
            <p className="text-[13px] text-muted-foreground max-w-[240px]">
              This section is being set up. Content will appear here soon.
            </p>
          </div>
        );
      } else {
        innerContent = (
          <div className="flex flex-col items-center justify-center py-16 text-center">
            <div className="w-12 h-12 rounded-2xl bg-secondary/60 flex items-center justify-center mb-4">
              <currentItem.icon className="w-6 h-6 text-muted-foreground/50" />
            </div>
            <h3 className="text-[17px] font-semibold text-foreground mb-1">
              {currentItem.label}
            </h3>
            <p className="text-[13px] text-muted-foreground max-w-[240px]">
              This section is being set up. Content will appear here soon.
            </p>
          </div>
        );
      }
    }

    return (
      <div className="w-full animate-fade-in">
        {/* Section header with segmented control */}
        <div className="mb-5 sm:mb-6">
          <h1 className="text-[28px] sm:text-[34px] font-bold tracking-tight text-foreground leading-tight mb-3">
            {currentItem.label}
          </h1>
          {currentItem.hasTabs && sectionTabs[activeSection] && (
            <SegmentedControl
              tabs={sectionTabs[activeSection]}
              activeTab={activeTab}
              onTabChange={handleTabChange}
            />
          )}
        </div>

        {/* Content */}
        {innerContent}
      </div>
    );
  };

  return (
    <div className="flex flex-col h-screen w-full bg-background">
      {/* Header — Apple Navigation Bar style */}
      <header className="h-12 sm:h-14 border-b-[0.5px] border-border/40 flex items-center justify-between px-4 sm:px-6 sticky top-0 z-30 bg-card/80 backdrop-blur-xl safe-area-top">
        <span className="text-[15px] sm:text-[17px] font-semibold tracking-tight text-foreground">
          EMS
        </span>
        <div className="flex items-center gap-2 sm:gap-3">
          {/* Tenant indicator — click to switch org */}
          <button
            onClick={() => selectTenant(null)}
            title="Switch organization"
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-secondary/40 hover:bg-secondary/60 transition-colors"
          >
            <Building2 className="w-3.5 h-3.5 text-primary" />
            <span className="text-[13px] font-medium text-foreground/70 truncate max-w-[80px] sm:max-w-[120px] hidden sm:inline">
              {currentTenant?.name}
            </span>
          </button>

          {/* User pill */}
          <div className="hidden sm:flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-secondary/40">
            <User className="w-3.5 h-3.5 text-primary/70" />
            <span className="text-[13px] text-muted-foreground truncate max-w-[100px]">
              {user?.first_name}
            </span>
          </div>

          {/* Sign out */}
          <Button
            variant="ghost"
            size="sm"
            onClick={handleLogout}
            className="text-destructive hover:bg-destructive/8 h-8 px-2 sm:px-3"
          >
            <LogOut className="w-4 h-4" />
            <span className="text-[13px] hidden sm:inline ml-1">Sign Out</span>
          </Button>
        </div>
      </header>

      {/* Mobile user menu overlay */}
      {showMobileUserMenu && (
        <>
          <div
            className="fixed inset-0 z-20 md:hidden"
            onClick={() => setShowMobileUserMenu(false)}
          />
          <div className="absolute top-12 right-4 z-30 md:hidden bg-card rounded-2xl shadow-soft-lg border-[0.5px] border-border/50 p-3 min-w-[200px] animate-scale-in">
            <div className="px-3 py-2 mb-1">
              <p className="text-[15px] font-semibold text-foreground">{user?.first_name} {user?.last_name}</p>
              <p className="text-[12px] text-muted-foreground">{currentTenant?.name}</p>
            </div>
            <div className="h-[0.5px] bg-border/50 my-1" />
            <button
              onClick={handleLogout}
              className="flex items-center gap-2 w-full px-3 py-2 rounded-lg text-destructive hover:bg-destructive/8 text-[14px] font-medium"
            >
              <LogOut className="w-4 h-4" />
              Sign Out
            </button>
          </div>
        </>
      )}

      {/* Main layout */}
      <div className="flex flex-1 min-h-0">
        {/* Desktop sidebar */}
        <div className="w-56 h-full flex-shrink-0 hidden md:block">
          <Sidebar
            items={sidebarItems}
            activeItem={activeSection}
            onItemClick={handleSectionChange}
          />
        </div>

        {/* Content area */}
        <div className="flex flex-col flex-1 min-w-0">
          <main className="flex-1 px-4 sm:px-6 lg:px-8 py-5 sm:py-6 overflow-auto pb-24 md:pb-6 bg-background">
            {renderContent()}
          </main>
        </div>
      </div>

      {/* Mobile bottom tab bar */}
      <BottomTabBar
        items={mobileTabItems}
        activeItem={activeSection}
        onItemClick={handleSectionChange}
        overflowItems={mobileOverflowItems}
      />
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
        <div className="flex flex-col items-center gap-3">
          <div className="w-10 h-10 rounded-2xl bg-primary flex items-center justify-center animate-pulse">
            <span className="text-white font-bold text-lg">E</span>
          </div>
          <div className="text-[13px] text-muted-foreground">Loading...</div>
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
