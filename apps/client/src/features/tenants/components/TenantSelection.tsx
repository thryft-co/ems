import React, { useState, useEffect } from "react";
import { Building2, LogOut, Plus, ArrowLeft, Check } from "lucide-react";
import { useAuth } from "@/features/auth/context/AuthContext";
import { TenantService } from "@/features/tenants/services/tenantService";
import { CreateTenantRequest, Tenant } from "@/features/tenants/types/tenant";
import { Badge } from "@/shared/ui/badge";
import { Button } from "@/shared/ui/button";
import { Input } from "@/shared/ui/input";
import { Label } from "@/shared/ui/label";

const TenantSelection: React.FC = () => {
  const { user, logout, isLoading: authLoading, joinTenant, createAndJoinTenant } = useAuth();
  const [tenants, setTenants] = useState<Tenant[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string>("");
  const [selectedTenant, setSelectedTenant] = useState<Tenant | null>(null);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [createFormData, setCreateFormData] = useState<CreateTenantRequest>({ name: "", subdomain: "" });
  const [createErrors, setCreateErrors] = useState<Record<string, string>>({});
  const [isCreating, setIsCreating] = useState(false);

  useEffect(() => {
    const fetchTenants = async () => {
      try {
        setIsLoading(true);
        setError("");
        const accessibleTenants = await TenantService.getAccessibleTenants();
        setTenants(accessibleTenants);
      } catch (error) {
        setError(error instanceof Error ? error.message : "Failed to load tenants");
      } finally {
        setIsLoading(false);
      }
    };
    fetchTenants();
  }, []);

  const handleTenantSelect = (tenant: Tenant) => setSelectedTenant(tenant);

  const handleConfirmSelection = async () => {
    if (!selectedTenant) return;
    try {
      setIsLoading(true);
      await joinTenant(selectedTenant.subdomain);
    } catch (error) {
      setError(error instanceof Error ? error.message : "Failed to join organization");
      setIsLoading(false);
    }
  };

  const handleCreateFormChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setCreateFormData((prev) => ({ ...prev, [name]: value }));
    if (createErrors[name]) setCreateErrors((prev) => ({ ...prev, [name]: "" }));
  };

  const validateCreateForm = (): boolean => {
    const newErrors: Record<string, string> = {};
    if (!createFormData.name) newErrors.name = "Organization name is required";
    else if (createFormData.name.length > 100) newErrors.name = "Name must be 100 characters or less";
    if (!createFormData.subdomain) newErrors.subdomain = "Subdomain is required";
    else if (createFormData.subdomain.length > 50) newErrors.subdomain = "Subdomain must be 50 characters or less";
    else if (!/^[a-z0-9-]+$/.test(createFormData.subdomain)) newErrors.subdomain = "Lowercase letters, numbers, and hyphens only";
    setCreateErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleCreateTenant = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!validateCreateForm()) return;
    try {
      setIsCreating(true);
      await createAndJoinTenant(createFormData.name, createFormData.subdomain);
    } catch (error) {
      setCreateErrors({ general: error instanceof Error ? error.message : "Failed to create organization" });
      setIsCreating(false);
    }
  };

  const handleLogout = async () => {
    try { await logout(); } catch (error) { console.error("Logout error:", error); }
  };

  const handleBackToList = () => {
    setShowCreateForm(false);
    setCreateFormData({ name: "", subdomain: "" });
    setCreateErrors({});
  };

  // Loading state
  if (isLoading || authLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="flex flex-col items-center gap-3">
          <div className="w-10 h-10 rounded-2xl bg-primary flex items-center justify-center animate-pulse">
            <Building2 className="w-5 h-5 text-white" />
          </div>
          <p className="text-[13px] text-muted-foreground">Loading organizations...</p>
        </div>
      </div>
    );
  }

  // Error state
  if (error && !showCreateForm) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background px-4">
        <div className="w-full max-w-sm text-center">
          <div className="mx-auto w-14 h-14 rounded-[16px] bg-primary flex items-center justify-center shadow-soft-md mb-5">
            <Building2 className="w-7 h-7 text-white" />
          </div>
          <h1 className="text-[22px] font-bold text-foreground mb-2">Get Started</h1>
          <p className="text-[14px] text-muted-foreground mb-6">
            {error.includes("fetch") || error.includes("network")
              ? "Unable to load organizations. Create a new one to get started."
              : "No organizations found. Create your first one."}
          </p>
          <div className="space-y-3">
            <Button onClick={() => setShowCreateForm(true)} className="w-full" size="lg">
              <Plus className="w-4 h-4 mr-2" />Create Organization
            </Button>
            <Button variant="outline" onClick={() => window.location.reload()} className="w-full">Retry</Button>
            <Button variant="ghost" onClick={handleLogout} className="w-full text-muted-foreground">
              <LogOut className="w-4 h-4 mr-2" />Sign Out
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-background px-4 sm:px-6 py-8">
      <div className="w-full max-w-lg">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-[28px] font-bold tracking-tight text-foreground">
              {showCreateForm ? "Create Organization" : "Select Organization"}
            </h1>
            <p className="text-[14px] text-muted-foreground mt-1">
              {showCreateForm ? "Set up your new organization" : `Welcome back, ${user?.first_name}`}
            </p>
          </div>
          <Button variant="ghost" size="sm" onClick={handleLogout} className="text-muted-foreground">
            <LogOut className="w-4 h-4" />
          </Button>
        </div>

        {/* Content Card */}
        <div className="bg-card rounded-2xl border-[0.5px] border-border/60 shadow-soft">
          {showCreateForm ? (
            <div className="p-5 sm:p-6">
              <button onClick={handleBackToList} className="flex items-center gap-1.5 text-[14px] text-primary font-medium mb-5 hover:text-primary/80 transition-colors">
                <ArrowLeft className="w-4 h-4" />Back
              </button>
              <form onSubmit={handleCreateTenant} className="space-y-4">
                <div>
                  <Label htmlFor="name">Organization Name</Label>
                  <Input id="name" name="name" type="text" value={createFormData.name} onChange={handleCreateFormChange} placeholder="Your organization name" className={createErrors.name ? "ring-2 ring-destructive/30" : ""} disabled={isCreating} />
                  {createErrors.name && <p className="mt-1.5 text-[13px] text-destructive">{createErrors.name}</p>}
                </div>
                <div>
                  <Label htmlFor="subdomain">Subdomain</Label>
                  <Input id="subdomain" name="subdomain" type="text" value={createFormData.subdomain} onChange={handleCreateFormChange} placeholder="your-organization" className={createErrors.subdomain ? "ring-2 ring-destructive/30" : ""} disabled={isCreating} />
                  {createErrors.subdomain && <p className="mt-1.5 text-[13px] text-destructive">{createErrors.subdomain}</p>}
                  <p className="mt-1.5 text-[12px] text-muted-foreground/60">Lowercase letters, numbers, and hyphens only.</p>
                </div>
                {createErrors.general && <p className="text-[13px] text-destructive text-center">{createErrors.general}</p>}
                <Button type="submit" className="w-full" size="lg" disabled={isCreating}>
                  {isCreating ? "Creating..." : "Create Organization"}
                </Button>
              </form>
            </div>
          ) : (
            <div>
              {tenants.length === 0 ? (
                <div className="text-center py-12 px-6">
                  <div className="w-12 h-12 rounded-2xl bg-secondary/60 flex items-center justify-center mx-auto mb-4">
                    <Building2 className="w-6 h-6 text-muted-foreground/50" />
                  </div>
                  <h3 className="text-[17px] font-semibold mb-2">No Organizations</h3>
                  <p className="text-[13px] text-muted-foreground mb-5">Create your first organization to get started.</p>
                  <Button onClick={() => setShowCreateForm(true)}>
                    <Plus className="w-4 h-4 mr-2" />Create Organization
                  </Button>
                </div>
              ) : (
                <div>
                  {/* Create new button */}
                  <div className="flex justify-end px-5 sm:px-6 pt-5 sm:pt-6">
                    <Button variant="outline" size="sm" onClick={() => setShowCreateForm(true)}>
                      <Plus className="w-3.5 h-3.5 mr-1.5" />New
                    </Button>
                  </div>

                  {/* Tenant list — Apple Settings style rows */}
                  <div className="px-3 sm:px-4 pb-3">
                    {tenants.map((tenant, index) => (
                      <button
                        key={tenant.id}
                        className={`flex items-center gap-3 w-full px-3 py-3.5 rounded-xl text-left transition-all ${
                          selectedTenant?.id === tenant.id
                            ? "bg-primary/8"
                            : "hover:bg-secondary/40"
                        }`}
                        onClick={() => handleTenantSelect(tenant)}
                      >
                        <div className={`w-9 h-9 rounded-[10px] flex items-center justify-center flex-shrink-0 ${
                          selectedTenant?.id === tenant.id ? "bg-primary/15" : "bg-secondary/60"
                        }`}>
                          <Building2 className={`w-4 h-4 ${selectedTenant?.id === tenant.id ? "text-primary" : "text-muted-foreground/60"}`} />
                        </div>
                        <div className="flex-1 min-w-0">
                          <p className="text-[15px] font-medium text-foreground truncate">{tenant.name}</p>
                          <p className="text-[12px] text-muted-foreground/60">{tenant.subdomain}</p>
                        </div>
                        <div className="flex items-center gap-2 flex-shrink-0">
                          {!tenant.is_active && <Badge variant="warning">Inactive</Badge>}
                          {selectedTenant?.id === tenant.id && (
                            <div className="w-5 h-5 rounded-full bg-primary flex items-center justify-center animate-scale-in">
                              <Check className="w-3 h-3 text-white" strokeWidth={3} />
                            </div>
                          )}
                        </div>
                      </button>
                    ))}
                  </div>

                  {/* Continue button */}
                  <div className="px-5 sm:px-6 pb-5 sm:pb-6 pt-2">
                    <div className="h-[0.5px] bg-border/50 mb-4" />
                    <Button onClick={handleConfirmSelection} disabled={!selectedTenant || !selectedTenant.is_active} className="w-full" size="lg">
                      {selectedTenant ? `Continue to ${selectedTenant.name}` : "Select an organization"}
                    </Button>
                    {selectedTenant && !selectedTenant.is_active && (
                      <p className="mt-2 text-[12px] text-warning text-center">This organization is inactive.</p>
                    )}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default TenantSelection;
