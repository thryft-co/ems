import React, { useState, useEffect } from "react";
import { Button } from "@/shared/ui/button";
import { Input } from "@/shared/ui/input";
import { Label } from "@/shared/ui/label";
import { Textarea } from "@/shared/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/shared/ui/select";
import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
import { Badge } from "@/shared/ui/badge";
import {
  ArrowLeft,
  Save,
  Eye,
  Edit,
  Calendar,
  User,
  Building,
} from "lucide-react";
import {
  PersonType,
  PersonFormData,
  PersonDetailResponse,
} from "@/features/persons/types/person";
import {
  createPerson,
  updatePerson,
  getPersonById,
} from "@/features/persons/services/personService";

const isValidUUID = (uuid: string): boolean => {
  const uuidRegex =
    /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
  return uuidRegex.test(uuid);
};

const formatAsUUID = (value: string): string => {
  // Remove any non-alphanumeric characters
  const cleaned = value.replace(/[^a-fA-F0-9]/g, "");

  // If it's already 32 characters, format as UUID
  if (cleaned.length === 32) {
    return `${cleaned.slice(0, 8)}-${cleaned.slice(8, 12)}-${cleaned.slice(12, 16)}-${cleaned.slice(16, 20)}-${cleaned.slice(20, 32)}`;
  }

  return value;
};

interface PersonFormViewProps {
  personId?: string;
  personType?: PersonType;
  mode: "view" | "edit" | "create";
  onSave?: () => void;
  onBack: () => void;
}

type FormState = {
  name: string;
  email: string;
  phone: string;
  person_type: PersonType;
  role: string;
  global_access: string;
  is_active: boolean;
  // Internal specific
  department: string;
  position: string;
  employee_id: string;
  hire_date: string;
  // Customer specific
  company: string;
  industry: string;
  customer_since: string;
  account_manager_id: string;
  // Vendor specific
  service_type: string;
  contract_start: string;
  contract_end: string;
  // Distributor specific
  territory: string;
  distribution_tier: string;
  commission_rate: string;
};

const PersonFormView: React.FC<PersonFormViewProps> = ({
  personId,
  personType = "internal",
  mode,
  onSave,
  onBack,
}) => {
  const [formData, setFormData] = useState<FormState>({
    name: "",
    email: "",
    phone: "",
    person_type: personType,
    role: "user",
    global_access: '["standard"]',
    is_active: true,
    // Internal specific
    department: "",
    position: "",
    employee_id: "",
    hire_date: "",
    // Customer specific
    company: "",
    industry: "",
    customer_since: "",
    account_manager_id: "",
    // Vendor specific
    service_type: "",
    contract_start: "",
    contract_end: "",
    // Distributor specific
    territory: "",
    distribution_tier: "",
    commission_rate: "",
  });

  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [personData, setPersonData] = useState<PersonDetailResponse | null>(
    null,
  );

  useEffect(() => {
    if (personId && (mode === "view" || mode === "edit")) {
      const fetchPersonData = async () => {
        try {
          setLoading(true);
          const data = await getPersonById(personId);
          setPersonData(data);

          // Populate form data
          setFormData({
            name: data.name || "",
            email: data.email || "",
            phone: data.phone || "",
            person_type: data.person_type,
            role: "user", // This would come from tenant_person relationship
            global_access: JSON.stringify(data.global_access || ["standard"]),
            is_active: data.is_active ?? true,
            // Internal specific
            department: data.internal?.department || "",
            position: data.internal?.position || "",
            employee_id: data.internal?.employee_id || "",
            hire_date: data.internal?.hire_date
              ? data.internal.hire_date.split("T")[0]
              : "",
            // Customer specific
            company: data.customer?.company || "",
            industry: data.customer?.industry || "",
            customer_since: data.customer?.customer_since
              ? data.customer.customer_since.split("T")[0]
              : "",
            account_manager_id: data.customer?.account_manager_id || "",
            // Vendor specific
            service_type: data.vendor?.service_type || "",
            contract_start: data.vendor?.contract_start
              ? data.vendor.contract_start.split("T")[0]
              : "",
            contract_end: data.vendor?.contract_end
              ? data.vendor.contract_end.split("T")[0]
              : "",
            // Distributor specific
            territory: data.distributor?.territory || "",
            distribution_tier: data.distributor?.distribution_tier || "",
            commission_rate: data.distributor?.commission_rate || "",
          });
        } catch (err) {
          setError("Failed to fetch person data");
          console.error("Error fetching person:", err);
        } finally {
          setLoading(false);
        }
      };

      fetchPersonData();
    }
  }, [personId, mode]);

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>,
  ): void => {
    const { name, value, type } = e.target;

    if (type === "checkbox") {
      const checked = (e.target as HTMLInputElement).checked;
      setFormData((prev) => ({
        ...prev,
        [name]: checked,
      }));
    } else if (name === "account_manager_id") {
      // Format UUID fields
      const formattedValue = formatAsUUID(value);
      setFormData((prev) => ({
        ...prev,
        [name]: formattedValue,
      }));
    } else {
      setFormData((prev) => ({
        ...prev,
        [name]: value,
      }));
    }
  };

  const handleSelectChange = (name: string, value: string | boolean): void => {
    setFormData((prev) => ({
      ...prev,
      [name]: value,
    }));
  };

  const handleSubmit = async (
    e: React.FormEvent<HTMLFormElement>,
  ): Promise<void> => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      // Validate required fields
      if (!formData.name.trim()) {
        throw new Error("Name is required");
      }
      if (!formData.email.trim()) {
        throw new Error("Email is required");
      }

      // Validate person type specific required fields
      if (formData.person_type === "internal") {
        if (!formData.department.trim()) {
          throw new Error("Department is required for internal users");
        }
        if (!formData.position.trim()) {
          throw new Error("Position is required for internal users");
        }
        if (!formData.employee_id.trim()) {
          throw new Error("Employee ID is required for internal users");
        }
      } else if (
        formData.person_type === "vendor" ||
        formData.person_type === "distributor"
      ) {
        if (!formData.company.trim()) {
          throw new Error("Company is required for vendors and distributors");
        }
      }

      // Validate UUID fields
      if (
        formData.account_manager_id &&
        !isValidUUID(formData.account_manager_id)
      ) {
        throw new Error("Account Manager ID must be a valid UUID");
      }

      // Prepare data for submission
      const submitData: PersonFormData = {
        name: formData.name.trim(),
        email: formData.email.trim(),
        phone: formData.phone.trim() || undefined,
        person_type: formData.person_type,
        role: formData.role,
        global_access: formData.global_access,
        is_active: formData.is_active,
      };

      // Add type-specific fields
      if (formData.person_type === "internal") {
        submitData.department = formData.department.trim();
        submitData.position = formData.position.trim();
        submitData.employee_id = formData.employee_id.trim();
        if (formData.hire_date) {
          submitData.hire_date = formData.hire_date;
        }
      } else if (formData.person_type === "customer") {
        if (formData.company.trim())
          submitData.company = formData.company.trim();
        if (formData.industry.trim())
          submitData.industry = formData.industry.trim();
        if (formData.customer_since)
          submitData.customer_since = formData.customer_since;
        if (formData.account_manager_id.trim())
          submitData.account_manager_id = formData.account_manager_id.trim();
      } else if (formData.person_type === "vendor") {
        submitData.company = formData.company.trim();
        if (formData.service_type.trim())
          submitData.service_type = formData.service_type.trim();
        if (formData.contract_start)
          submitData.contract_start = formData.contract_start;
        if (formData.contract_end)
          submitData.contract_end = formData.contract_end;
      } else if (formData.person_type === "distributor") {
        submitData.company = formData.company.trim();
        if (formData.territory.trim())
          submitData.territory = formData.territory.trim();
        if (formData.distribution_tier.trim())
          submitData.distribution_tier = formData.distribution_tier.trim();
        if (formData.commission_rate.trim())
          submitData.commission_rate = formData.commission_rate.trim();
      }

      if (mode === "create") {
        await createPerson(submitData);
      } else if (mode === "edit" && personId) {
        await updatePerson(personId, submitData);
      }

      if (onSave) {
        onSave();
      }
    } catch (err: any) {
      setError(err.message || "An error occurred while saving the person");
      console.error("Error saving person:", err);
    } finally {
      setLoading(false);
    }
  };

  const getTitle = () => {
    const typeLabel =
      formData.person_type.charAt(0).toUpperCase() +
      formData.person_type.slice(1);
    if (mode === "create") {
      return `Create ${typeLabel}`;
    } else if (mode === "edit") {
      return `Edit ${typeLabel}`;
    } else {
      return `View ${typeLabel}`;
    }
  };

  const getPersonTypeColor = (type: string): string => {
    const typeColors: { [key: string]: string } = {
      internal: "bg-blue-100 text-blue-800 border-blue-200",
      customer: "bg-green-100 text-green-800 border-green-200",
      vendor: "bg-purple-100 text-purple-800 border-purple-200",
      distributor: "bg-orange-100 text-orange-800 border-orange-200",
    };
    return typeColors[type] || "bg-gray-100 text-gray-800 border-gray-200";
  };

  const renderTypeSpecificFields = () => {
    switch (formData.person_type) {
      case "internal":
        return (
          <>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <Label htmlFor="department">Department *</Label>
                <Input
                  id="department"
                  name="department"
                  value={formData.department}
                  onChange={handleChange}
                  disabled={mode === "view"}
                  required
                />
              </div>
              <div>
                <Label htmlFor="position">Position *</Label>
                <Input
                  id="position"
                  name="position"
                  value={formData.position}
                  onChange={handleChange}
                  disabled={mode === "view"}
                  required
                />
              </div>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <Label htmlFor="employee_id">Employee ID *</Label>
                <Input
                  id="employee_id"
                  name="employee_id"
                  value={formData.employee_id}
                  onChange={handleChange}
                  disabled={mode === "view"}
                  required
                />
              </div>
              <div>
                <Label htmlFor="hire_date">Hire Date</Label>
                <Input
                  id="hire_date"
                  name="hire_date"
                  type="date"
                  value={formData.hire_date}
                  onChange={handleChange}
                  disabled={mode === "view"}
                />
              </div>
            </div>
          </>
        );

      case "customer":
        return (
          <>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <Label htmlFor="company">Company</Label>
                <Input
                  id="company"
                  name="company"
                  value={formData.company}
                  onChange={handleChange}
                  disabled={mode === "view"}
                />
              </div>
              <div>
                <Label htmlFor="industry">Industry</Label>
                <Input
                  id="industry"
                  name="industry"
                  value={formData.industry}
                  onChange={handleChange}
                  disabled={mode === "view"}
                />
              </div>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <Label htmlFor="customer_since">Customer Since</Label>
                <Input
                  id="customer_since"
                  name="customer_since"
                  type="date"
                  value={formData.customer_since}
                  onChange={handleChange}
                  disabled={mode === "view"}
                />
              </div>
              <div>
                <Label htmlFor="account_manager_id">Account Manager ID</Label>
                <Input
                  id="account_manager_id"
                  name="account_manager_id"
                  value={formData.account_manager_id}
                  onChange={handleChange}
                  disabled={mode === "view"}
                  placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                />
              </div>
            </div>
          </>
        );

      case "vendor":
        return (
          <>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <Label htmlFor="company">Company *</Label>
                <Input
                  id="company"
                  name="company"
                  value={formData.company}
                  onChange={handleChange}
                  disabled={mode === "view"}
                  required
                />
              </div>
              <div>
                <Label htmlFor="service_type">Service Type</Label>
                <Input
                  id="service_type"
                  name="service_type"
                  value={formData.service_type}
                  onChange={handleChange}
                  disabled={mode === "view"}
                />
              </div>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <Label htmlFor="contract_start">Contract Start</Label>
                <Input
                  id="contract_start"
                  name="contract_start"
                  type="date"
                  value={formData.contract_start}
                  onChange={handleChange}
                  disabled={mode === "view"}
                />
              </div>
              <div>
                <Label htmlFor="contract_end">Contract End</Label>
                <Input
                  id="contract_end"
                  name="contract_end"
                  type="date"
                  value={formData.contract_end}
                  onChange={handleChange}
                  disabled={mode === "view"}
                />
              </div>
            </div>
          </>
        );

      case "distributor":
        return (
          <>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <Label htmlFor="company">Company *</Label>
                <Input
                  id="company"
                  name="company"
                  value={formData.company}
                  onChange={handleChange}
                  disabled={mode === "view"}
                  required
                />
              </div>
              <div>
                <Label htmlFor="territory">Territory</Label>
                <Input
                  id="territory"
                  name="territory"
                  value={formData.territory}
                  onChange={handleChange}
                  disabled={mode === "view"}
                />
              </div>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <Label htmlFor="distribution_tier">Distribution Tier</Label>
                <Select
                  value={formData.distribution_tier}
                  onValueChange={(value) =>
                    handleSelectChange("distribution_tier", value)
                  }
                  disabled={mode === "view"}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select tier" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="tier1">Tier 1</SelectItem>
                    <SelectItem value="tier2">Tier 2</SelectItem>
                    <SelectItem value="tier3">Tier 3</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label htmlFor="commission_rate">Commission Rate (%)</Label>
                <Input
                  id="commission_rate"
                  name="commission_rate"
                  value={formData.commission_rate}
                  onChange={handleChange}
                  disabled={mode === "view"}
                  placeholder="e.g., 5.5"
                />
              </div>
            </div>
          </>
        );

      default:
        return null;
    }
  };

  if (loading && (mode === "view" || mode === "edit")) {
    return (
      <div className="flex justify-center items-center py-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <Button variant="outline" onClick={onBack} size="sm">
            <ArrowLeft className="h-4 w-4 mr-2" />
            Back
          </Button>
          <div className="flex items-center space-x-2">
            <h1 className="text-2xl font-bold">{getTitle()}</h1>
            <Badge
              className={`${getPersonTypeColor(formData.person_type)} text-xs font-medium`}
            >
              {formData.person_type.charAt(0).toUpperCase() +
                formData.person_type.slice(1)}
            </Badge>
          </div>
        </div>
        {mode === "view" && personData && (
          <div className="flex items-center space-x-2 text-sm text-gray-600">
            <Calendar className="h-4 w-4" />
            <span>
              Created:{" "}
              {new Date(personData.created_at || "").toLocaleDateString()}
            </span>
          </div>
        )}
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
          {error}
        </div>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <User className="h-5 w-5" />
            <span>Person Information</span>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-6">
            {/* Basic Information */}
            <div className="space-y-4">
              <h3 className="text-lg font-medium">Basic Information</h3>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="name">Name *</Label>
                  <Input
                    id="name"
                    name="name"
                    value={formData.name}
                    onChange={handleChange}
                    disabled={mode === "view"}
                    required
                  />
                </div>
                <div>
                  <Label htmlFor="email">Email *</Label>
                  <Input
                    id="email"
                    name="email"
                    type="email"
                    value={formData.email}
                    onChange={handleChange}
                    disabled={mode === "view"}
                    required
                  />
                </div>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="phone">Phone</Label>
                  <Input
                    id="phone"
                    name="phone"
                    value={formData.phone}
                    onChange={handleChange}
                    disabled={mode === "view"}
                  />
                </div>
                <div>
                  <Label htmlFor="person_type">Person Type</Label>
                  <Select
                    value={formData.person_type}
                    onValueChange={(value) =>
                      handleSelectChange("person_type", value as PersonType)
                    }
                    disabled={mode === "view" || mode === "edit"}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="internal">Internal</SelectItem>
                      <SelectItem value="customer">Customer</SelectItem>
                      <SelectItem value="vendor">Vendor</SelectItem>
                      <SelectItem value="distributor">Distributor</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </div>

            {/* Type-specific fields */}
            <div className="space-y-4">
              <h3 className="text-lg font-medium">
                {formData.person_type.charAt(0).toUpperCase() +
                  formData.person_type.slice(1)}{" "}
                Details
              </h3>
              {renderTypeSpecificFields()}
            </div>

            {/* Form Actions */}
            {mode !== "view" && (
              <div className="flex justify-end space-x-2 pt-6 border-t">
                <Button type="button" variant="outline" onClick={onBack}>
                  Cancel
                </Button>
                <Button type="submit" disabled={loading}>
                  {loading ? (
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                  ) : (
                    <Save className="h-4 w-4 mr-2" />
                  )}
                  {mode === "create" ? "Create" : "Save Changes"}
                </Button>
              </div>
            )}
          </form>
        </CardContent>
      </Card>
    </div>
  );
};

export default PersonFormView;
