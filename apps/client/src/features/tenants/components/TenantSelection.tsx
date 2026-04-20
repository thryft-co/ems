import React, { useState, useEffect } from "react";
import { Building2, Users, LogOut, Plus, ArrowLeft } from "lucide-react";
import { useAuth } from "@/features/auth/context/AuthContext";
import { TenantService } from "@/features/tenants/services/tenantService";
import { CreateTenantRequest, Tenant } from "@/features/tenants/types/tenant";
import { Badge } from "@/shared/ui/badge";
import { Button } from "@/shared/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
import { Input } from "@/shared/ui/input";
import { Label } from "@/shared/ui/label";

const TenantSelection: React.FC = () => {
  const {
    user,
    logout,
    isLoading: authLoading,
    joinTenant,
    createAndJoinTenant,
  } = useAuth();
  const [tenants, setTenants] = useState<Tenant[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string>("");
  const [selectedTenant, setSelectedTenant] = useState<Tenant | null>(null);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [createFormData, setCreateFormData] = useState<CreateTenantRequest>({
    name: "",
    subdomain: "",
  });
  const [createErrors, setCreateErrors] = useState<Record<string, string>>({});
  const [isCreating, setIsCreating] = useState(false);

  // Fetch available tenants on component mount
  useEffect(() => {
    const fetchTenants = async () => {
      try {
        setIsLoading(true);
        setError("");

        const accessibleTenants = await TenantService.getAccessibleTenants();
        setTenants(accessibleTenants);
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : "Failed to load tenants";
        setError(errorMessage);
      } finally {
        setIsLoading(false);
      }
    };

    fetchTenants();
  }, []);

  // Handle tenant selection
  const handleTenantSelect = (tenant: Tenant) => {
    setSelectedTenant(tenant);
  };

  // Handle confirm selection (join existing tenant)
  const handleConfirmSelection = async () => {
    if (!selectedTenant) return;

    try {
      setIsLoading(true);
      await joinTenant(selectedTenant.subdomain);
      // AuthContext will handle the navigation after successful join
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Failed to join organization";
      setError(errorMessage);
      setIsLoading(false);
    }
  };

  // Handle create form input changes
  const handleCreateFormChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setCreateFormData((prev) => ({
      ...prev,
      [name]: value,
    }));

    // Clear field error when user starts typing
    if (createErrors[name]) {
      setCreateErrors((prev) => ({
        ...prev,
        [name]: "",
      }));
    }
  };

  // Validate create form
  const validateCreateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    // Organization name validation
    if (!createFormData.name) {
      newErrors.name = "Organization name is required";
    } else if (createFormData.name.length > 100) {
      newErrors.name = "Organization name must be 100 characters or less";
    }

    // Subdomain validation
    if (!createFormData.subdomain) {
      newErrors.subdomain = "Subdomain is required";
    } else if (createFormData.subdomain.length > 50) {
      newErrors.subdomain = "Subdomain must be 50 characters or less";
    } else if (!/^[a-z0-9-]+$/.test(createFormData.subdomain)) {
      newErrors.subdomain =
        "Subdomain can only contain lowercase letters, numbers, and hyphens";
    }

    setCreateErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  // Handle create new tenant
  const handleCreateTenant = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validateCreateForm()) {
      return;
    }

    try {
      setIsCreating(true);
      await createAndJoinTenant(createFormData.name, createFormData.subdomain);
      // AuthContext will handle the navigation after successful creation
    } catch (error) {
      const errorMessage =
        error instanceof Error
          ? error.message
          : "Failed to create organization";
      setCreateErrors({ general: errorMessage });
      setIsCreating(false);
    }
  };

  // Handle logout
  const handleLogout = async () => {
    try {
      await logout();
    } catch (error) {
      console.error("Logout error:", error);
    }
  };

  // Handle back to tenant list
  const handleBackToList = () => {
    setShowCreateForm(false);
    setCreateFormData({ name: "", subdomain: "" });
    setCreateErrors({});
  };

  // Loading state
  if (isLoading || authLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background via-background to-muted/50">
        <Card className="w-full max-w-md shadow-soft-lg">
          <CardContent className="pt-6">
            <div className="text-center">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-primary to-primary/70 flex items-center justify-center shadow-soft mx-auto mb-4 animate-pulse">
                <Building2 className="w-5 h-5 text-white" />
              </div>
              <p className="text-muted-foreground">Loading organizations...</p>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  // Error state - show create option alongside error (unless user is creating)
  if (error && !showCreateForm) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background via-background to-muted/50 py-12 px-4">
        <Card className="w-full max-w-md shadow-soft-lg">
          <CardHeader className="text-center pb-2">
            <div className="mx-auto w-12 h-12 rounded-2xl bg-gradient-to-br from-primary to-primary/70 flex items-center justify-center shadow-soft mb-4">
              <Building2 className="w-6 h-6 text-white" />
            </div>
            <CardTitle className="text-xl font-semibold tracking-tight">
              Get Started
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-center space-y-4">
              <p className="text-muted-foreground text-sm">
                {error.includes("fetch") || error.includes("network")
                  ? "Unable to load organizations. You can create a new one to get started."
                  : "No existing organizations found. Create your first organization to get started."}
              </p>
              <div className="space-y-3">
                <Button
                  onClick={() => setShowCreateForm(true)}
                  className="w-full"
                >
                  <Plus className="w-4 h-4 mr-2" />
                  Create Organization
                </Button>
                <Button
                  variant="outline"
                  onClick={() => window.location.reload()}
                  className="w-full"
                >
                  Retry Loading
                </Button>
                <Button
                  variant="ghost"
                  onClick={handleLogout}
                  className="w-full text-muted-foreground"
                >
                  <LogOut className="w-4 h-4 mr-2" />
                  Sign Out
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background via-background to-muted/50 py-12 px-4 sm:px-6 lg:px-8">
      <Card className="w-full max-w-2xl shadow-soft-lg">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="text-2xl font-bold">
                {showCreateForm ? "Create Organization" : "Select Organization"}
              </CardTitle>
              <p className="text-muted-foreground">
                {showCreateForm
                  ? "Set up your new organization"
                  : `Welcome back, ${user?.first_name}! Choose your organization to continue.`}
              </p>
            </div>
            <Button variant="outline" size="sm" onClick={handleLogout}>
              <LogOut className="w-4 h-4 mr-2" />
              Sign Out
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {showCreateForm ? (
            // Create new tenant form
            <div className="space-y-4">
              <div className="flex items-center mb-4">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleBackToList}
                  className="p-2"
                >
                  <ArrowLeft className="w-4 h-4" />
                </Button>
                <span className="ml-2 text-sm text-muted-foreground">
                  Back to organization list
                </span>
              </div>

              <form onSubmit={handleCreateTenant} className="space-y-4">
                <div>
                  <Label htmlFor="name">Organization Name</Label>
                  <Input
                    id="name"
                    name="name"
                    type="text"
                    value={createFormData.name}
                    onChange={handleCreateFormChange}
                    placeholder="Your organization name"
                    className={createErrors.name ? "border-red-500" : ""}
                    disabled={isCreating}
                  />
                  {createErrors.name && (
                    <p className="mt-1 text-sm text-red-600">
                      {createErrors.name}
                    </p>
                  )}
                </div>

                <div>
                  <Label htmlFor="subdomain">Subdomain</Label>
                  <Input
                    id="subdomain"
                    name="subdomain"
                    type="text"
                    value={createFormData.subdomain}
                    onChange={handleCreateFormChange}
                    placeholder="your-organization"
                    className={createErrors.subdomain ? "border-red-500" : ""}
                    disabled={isCreating}
                  />
                  {createErrors.subdomain && (
                    <p className="mt-1 text-sm text-red-600">
                      {createErrors.subdomain}
                    </p>
                  )}
                  <p className="mt-1 text-xs text-muted-foreground">
                    This will be your unique identifier. Use only lowercase
                    letters, numbers, and hyphens.
                  </p>
                </div>

                {createErrors.general && (
                  <div className="p-3 bg-red-100 border border-red-400 text-red-700 rounded">
                    {createErrors.general}
                  </div>
                )}

                <Button type="submit" className="w-full" disabled={isCreating}>
                  {isCreating
                    ? "Creating Organization..."
                    : "Create Organization"}
                </Button>
              </form>
            </div>
          ) : (
            // Tenant selection
            <div className="space-y-4">
              {tenants.length === 0 ? (
                <div className="text-center py-12">
                  <Building2 className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
                  <h3 className="text-lg font-semibold mb-2">
                    No Organizations Available
                  </h3>
                  <p className="text-muted-foreground mb-4">
                    You don't have access to any organizations yet. Create your
                    first one to get started.
                  </p>
                  <Button onClick={() => setShowCreateForm(true)}>
                    <Plus className="w-4 h-4 mr-2" />
                    Create Organization
                  </Button>
                </div>
              ) : (
                <div className="space-y-4">
                  {/* Create New Button */}
                  <div className="flex justify-end">
                    <Button
                      variant="outline"
                      onClick={() => setShowCreateForm(true)}
                    >
                      <Plus className="w-4 h-4 mr-2" />
                      Create New Organization
                    </Button>
                  </div>

                  {/* Tenant List */}
                  <div className="grid gap-3">
                    {tenants.map((tenant) => (
                      <div
                        key={tenant.id}
                        className={`p-4 border rounded-lg cursor-pointer transition-all hover:bg-gray-50 ${
                          selectedTenant?.id === tenant.id
                            ? "border-primary bg-primary/5 ring-2 ring-primary/20"
                            : "border-gray-200"
                        }`}
                        onClick={() => handleTenantSelect(tenant)}
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex items-center space-x-3">
                            <div className="p-2 bg-primary/10 rounded-lg">
                              <Building2 className="w-5 h-5 text-primary" />
                            </div>
                            <div>
                              <h3 className="font-semibold text-lg">
                                {tenant.name}
                              </h3>
                              <p className="text-sm text-muted-foreground">
                                {tenant.subdomain}
                              </p>
                            </div>
                          </div>
                          <div className="flex items-center space-x-2">
                            {tenant.is_active ? (
                              <Badge
                                variant="default"
                                className="bg-green-100 text-green-800"
                              >
                                Active
                              </Badge>
                            ) : (
                              <Badge variant="secondary">Inactive</Badge>
                            )}
                            {selectedTenant?.id === tenant.id && (
                              <div className="w-4 h-4 rounded-full bg-primary flex items-center justify-center">
                                <div className="w-2 h-2 rounded-full bg-white"></div>
                              </div>
                            )}
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>

                  {/* Continue Button */}
                  <div className="pt-4 border-t">
                    <Button
                      onClick={handleConfirmSelection}
                      disabled={!selectedTenant || !selectedTenant.is_active}
                      className="w-full"
                    >
                      {selectedTenant
                        ? `Continue to ${selectedTenant.name}`
                        : "Select an organization to continue"}
                    </Button>
                    {selectedTenant && !selectedTenant.is_active && (
                      <p className="mt-2 text-sm text-amber-600 text-center">
                        This organization is currently inactive. Please contact
                        your administrator.
                      </p>
                    )}
                  </div>
                </div>
              )}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
};

export default TenantSelection;
