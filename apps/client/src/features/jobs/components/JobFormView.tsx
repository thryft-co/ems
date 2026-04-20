import React, { useState, useEffect } from "react";
import { Button } from "@/shared/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/shared/ui/card";
import { Input } from "@/shared/ui/input";
import { Label } from "@/shared/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/shared/ui/select";
import { Textarea } from "@/shared/ui/textarea";
import { Switch } from "@/shared/ui/switch";
import { Badge } from "@/shared/ui/badge";
import {
  createJob,
  updateJob,
  getJobById,
  getJobHistory,
} from "@/features/jobs/services/jobService";
import { ArrowLeft, Calendar, Clock } from "lucide-react";
import {
  JobType,
  JobPriority,
  JobStatus,
  JobFormData,
  JobDetailResponse,
  JobHistory,
} from "@/features/jobs/types/job";
import { formatDate } from "@/shared/utils";

// UUID validation regex
const UUID_REGEX =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

// Validate UUID format
const isValidUUID = (uuid: string): boolean => {
  return uuid === "" || UUID_REGEX.test(uuid);
};

// Format a string as UUID if possible
const formatAsUUID = (value: string): string => {
  // If already valid, return as is
  if (isValidUUID(value)) return value;

  // Remove all non-hex characters
  const hexOnly = value.replace(/[^0-9a-f]/gi, "");

  // If we don't have enough characters for a UUID, return as is
  if (hexOnly.length < 32) return value;

  // Format as UUID
  return `${hexOnly.slice(0, 8)}-${hexOnly.slice(8, 12)}-${hexOnly.slice(12, 16)}-${hexOnly.slice(16, 20)}-${hexOnly.slice(20, 32)}`;
};

interface StatusColorsType {
  [key: string]: string;
}

interface PriorityColorsType {
  [key: string]: string;
}

const statusColors: StatusColorsType = {
  pending: "bg-yellow-200 text-yellow-800",
  in_progress: "bg-blue-200 text-blue-800",
  on_hold: "bg-orange-200 text-orange-800",
  completed: "bg-green-200 text-green-800",
  cancelled: "bg-red-200 text-red-800",
};

const priorityColors: PriorityColorsType = {
  low: "bg-slate-200 text-slate-800",
  normal: "bg-blue-200 text-blue-800",
  high: "bg-amber-200 text-amber-800",
  urgent: "bg-red-200 text-red-800",
};

// Define component props
interface JobFormViewProps {
  jobId?: string;
  jobType?: JobType;
  mode: "view" | "edit" | "create";
  onSave?: () => void;
  onBack: () => void;
}

// Define the type for form state
type FormState = {
  job_number: string;
  item_id: string;
  quantity: number;
  assigned_user_id: string;
  supervisor_id: string;
  customer_id: string;
  job_type: JobType;
  priority: JobPriority;
  status: JobStatus;
  due_date: string;
  comments: string;
  // Manufacturing specific
  production_line: string;
  batch_number: string;
  raw_materials: string;
  quality_check_required: boolean;
  // QA specific
  test_procedure: string;
  inspection_type: string;
  pass_criteria: string;
  qa_results: string;
  // Service specific
  service_type: string;
  problem_description: string;
  diagnosis: string;
  solution: string;
  parts_replaced: string;
};

const JobFormView: React.FC<JobFormViewProps> = ({
  jobId,
  jobType = "manufacturing",
  mode,
  onSave,
  onBack,
}) => {
  const isViewMode = mode === "view";
  const isEditMode = mode === "edit";
  const isCreateMode = mode === "create";

  const [job, setJob] = useState<JobDetailResponse | null>(null);
  const [history, setHistory] = useState<JobHistory[]>([]);
  const [formData, setFormData] = useState<FormState>({
    job_number: "",
    item_id: "",
    quantity: 1,
    assigned_user_id: "",
    supervisor_id: "",
    customer_id: "",
    job_type: jobType,
    priority: "normal",
    status: "pending",
    due_date: "",
    comments: "",
    // Manufacturing specific
    production_line: "",
    batch_number: "",
    raw_materials: "[]",
    quality_check_required: true,
    // QA specific
    test_procedure: "",
    inspection_type: "",
    pass_criteria: "[]",
    qa_results: "{}",
    // Service specific
    service_type: "",
    problem_description: "",
    diagnosis: "",
    solution: "",
    parts_replaced: "[]",
  });

  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [validationErrors, setValidationErrors] = useState<{
    [key: string]: string;
  }>({});

  useEffect(() => {
    const fetchJobData = async () => {
      if (!jobId) {
        setLoading(false);
        return;
      }

      try {
        setLoading(true);
        setError(null);

        const jobData = await getJobById(jobId);
        setJob(jobData);

        if (isViewMode) {
          // In view mode, also fetch history
          const historyData = await getJobHistory(jobId);
          setHistory(historyData);
        }

        // Map job data to form state
        const updatedFormData: FormState = {
          // Initialize with default values for all fields
          job_number: jobData.job_number || "",
          item_id: jobData.item_id || "",
          quantity: jobData.quantity || 1,
          assigned_user_id: jobData.assigned_user_id || "",
          supervisor_id: jobData.supervisor_id || "",
          customer_id: jobData.customer_id || "",
          job_type: jobData.job_type || jobType,
          priority: jobData.priority || "normal",
          status: jobData.status || "pending",
          due_date: jobData.due_date ? jobData.due_date.split("T")[0] : "",
          comments: jobData.comments || "",

          // Default values for manufacturing specific fields
          production_line: "",
          batch_number: "",
          raw_materials: "[]",
          quality_check_required: true,

          // Default values for QA specific fields
          test_procedure: "",
          inspection_type: "",
          pass_criteria: "[]",
          qa_results: "{}",

          // Default values for service specific fields
          service_type: "",
          problem_description: "",
          diagnosis: "",
          solution: "",
          parts_replaced: "[]",
        };

        // Add type-specific fields
        if (jobData.job_type === "manufacturing" && jobData.manufacturing) {
          updatedFormData.production_line =
            jobData.manufacturing.production_line || "";
          updatedFormData.batch_number =
            jobData.manufacturing.batch_number || "";
          updatedFormData.quality_check_required =
            jobData.manufacturing.quality_check_required ?? true;
          updatedFormData.raw_materials = jobData.manufacturing.raw_materials
            ? JSON.stringify(jobData.manufacturing.raw_materials)
            : "[]";
        } else if (jobData.job_type === "qa" && jobData.qa) {
          updatedFormData.test_procedure = jobData.qa.test_procedure || "";
          updatedFormData.inspection_type = jobData.qa.inspection_type || "";
          updatedFormData.pass_criteria = jobData.qa.pass_criteria
            ? JSON.stringify(jobData.qa.pass_criteria)
            : "[]";
          updatedFormData.qa_results = jobData.qa.qa_results
            ? JSON.stringify(jobData.qa.qa_results)
            : "{}";
        } else if (jobData.job_type === "service" && jobData.service) {
          updatedFormData.service_type = jobData.service.service_type || "";
          updatedFormData.problem_description =
            jobData.service.problem_description || "";
          updatedFormData.diagnosis = jobData.service.diagnosis || "";
          updatedFormData.solution = jobData.service.solution || "";
          updatedFormData.parts_replaced = jobData.service.parts_replaced
            ? JSON.stringify(jobData.service.parts_replaced)
            : "[]";
        }

        setFormData(updatedFormData);
      } catch (err) {
        console.error("Failed to fetch job details:", err);
        setError("Failed to load job details. Please try again.");
      } finally {
        setLoading(false);
      }
    };

    fetchJobData();
  }, [jobId, jobType, isViewMode]);

  // Handle form field changes (only in edit/create mode)
  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>,
  ): void => {
    if (isViewMode) return;

    const { name, value, type } = e.target;
    let processedValue: string | number | boolean = value;

    // For number inputs, convert to number
    if (type === "number") {
      processedValue = value === "" ? 0 : Number(value);
    }

    // Format UUIDs if needed
    if (
      (name === "item_id" ||
        name === "assigned_user_id" ||
        name === "supervisor_id" ||
        name === "customer_id") &&
      value.length > 0
    ) {
      processedValue = formatAsUUID(value);
    }

    setFormData({ ...formData, [name]: processedValue });

    // Clear validation error when field is changed
    if (validationErrors[name]) {
      const newErrors = { ...validationErrors };
      delete newErrors[name];
      setValidationErrors(newErrors);
    }
  };

  // Handle select/dropdown changes
  const handleSelectChange = (name: string, value: string | boolean): void => {
    if (isViewMode) return;
    setFormData({ ...formData, [name]: value });

    // Clear validation error when field is changed
    if (validationErrors[name]) {
      const newErrors = { ...validationErrors };
      delete newErrors[name];
      setValidationErrors(newErrors);
    }
  };

  // Form submission
  const handleSubmit = async (
    e: React.FormEvent<HTMLFormElement>,
  ): Promise<void> => {
    e.preventDefault();
    if (isViewMode) return;

    // Clear any previous messages
    setError(null);
    setSuccessMessage(null);

    // Validate form
    const errors: { [key: string]: string } = {};

    if (!formData.job_number.trim()) {
      errors.job_number = "Job number is required";
    }

    if (!formData.item_id.trim()) {
      errors.item_id = "Item ID is required";
    } else if (!isValidUUID(formData.item_id)) {
      errors.item_id = "Invalid UUID format";
    }

    if (formData.quantity <= 0) {
      errors.quantity = "Quantity must be greater than 0";
    }

    if (formData.assigned_user_id && !isValidUUID(formData.assigned_user_id)) {
      errors.assigned_user_id = "Invalid UUID format";
    }

    if (formData.supervisor_id && !isValidUUID(formData.supervisor_id)) {
      errors.supervisor_id = "Invalid UUID format";
    }

    if (formData.customer_id && !isValidUUID(formData.customer_id)) {
      errors.customer_id = "Invalid UUID format";
    }

    // Validate JSON fields
    try {
      if (formData.raw_materials) JSON.parse(formData.raw_materials);
      if (formData.pass_criteria) JSON.parse(formData.pass_criteria);
      if (formData.qa_results) JSON.parse(formData.qa_results);
      if (formData.parts_replaced) JSON.parse(formData.parts_replaced);
    } catch (err) {
      errors.json = "Invalid JSON format in one or more fields";
    }

    if (Object.keys(errors).length > 0) {
      setValidationErrors(errors);
      return;
    }

    try {
      setLoading(true);

      // Convert form data to API format
      const jobData: JobFormData = {
        ...formData,
        job_type: formData.job_type,
      };

      if (isEditMode && jobId) {
        console.log("Updating job with data:", jobData);
        await updateJob(jobId, jobData);
        setSuccessMessage("Job updated successfully!");
        // Delay before calling onSave to show the success message
        setTimeout(() => {
          if (onSave) onSave();
        }, 1500);
      } else if (isCreateMode) {
        await createJob(jobData);
        setSuccessMessage("Job created successfully!");
        // Delay before calling onSave to show the success message
        setTimeout(() => {
          if (onSave) onSave();
        }, 1500);
      }
    } catch (err: any) {
      console.error("Failed to save job:", err);
      setError(err.message || "Failed to save job. Please try again.");
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="text-center p-8">
        <div className="animate-spin h-8 w-8 border-2 border-primary border-t-transparent rounded-full mx-auto"></div>
        <p className="mt-2 text-muted-foreground">Loading...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div
        className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded"
        role="alert"
      >
        <p>{error}</p>
        <Button variant="outline" className="mt-4" onClick={onBack}>
          <ArrowLeft className="h-4 w-4 mr-2" />
          Back
        </Button>
      </div>
    );
  }

  // Get page title based on mode
  const getTitle = () => {
    if (isViewMode) return `Job Details: ${formData.job_number}`;
    if (isEditMode) return `Edit Job: ${formData.job_number}`;
    return `Create New ${formData.job_type.charAt(0).toUpperCase() + formData.job_type.slice(1)} Job`;
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <Button variant="outline" onClick={onBack}>
          <ArrowLeft className="h-4 w-4 mr-2" />
          Back
        </Button>
        <div className="flex gap-2 items-center">
          {isViewMode && (
            <Button
              onClick={onBack} // This will trigger closing the view mode in the parent
            >
              Edit Job
            </Button>
          )}
          {isViewMode && (
            <div className="flex gap-2">
              <Badge className={statusColors[formData.status] || "bg-gray-200"}>
                {formData.status?.toUpperCase() || "UNKNOWN"}
              </Badge>
              <Badge
                className={priorityColors[formData.priority] || "bg-gray-200"}
              >
                {formData.priority?.toUpperCase() || "NORMAL"}
              </Badge>
            </div>
          )}
        </div>
      </div>

      {successMessage && (
        <div
          className="bg-green-100 border border-green-400 text-green-700 px-4 py-3 rounded"
          role="alert"
        >
          <p>{successMessage}</p>
        </div>
      )}

      {error && (
        <div
          className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded"
          role="alert"
        >
          <p>{error}</p>
        </div>
      )}

      <form onSubmit={handleSubmit}>
        <Card>
          <CardHeader>
            <CardTitle>{getTitle()}</CardTitle>
            <CardDescription>
              {isViewMode
                ? `View details for ${formData.job_type} job`
                : isEditMode
                  ? `Update information for this ${formData.job_type} job`
                  : `Create a new ${formData.job_type} job`}
            </CardDescription>
          </CardHeader>

          <CardContent className="space-y-6">
            {/* Basic Information Section */}
            <div>
              <h3 className="text-lg font-medium border-b pb-2 mb-4">
                Basic Information
              </h3>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="job_number">Job Number</Label>
                  <Input
                    id="job_number"
                    name="job_number"
                    value={formData.job_number}
                    onChange={handleChange}
                    disabled={isViewMode || isEditMode}
                    required
                    className={
                      validationErrors.job_number ? "border-red-500" : ""
                    }
                  />
                  {validationErrors.job_number && (
                    <p className="text-red-500 text-sm">
                      {validationErrors.job_number}
                    </p>
                  )}
                </div>

                <div className="space-y-2">
                  <Label htmlFor="item_id">Item ID (UUID)</Label>
                  <Input
                    id="item_id"
                    name="item_id"
                    value={formData.item_id}
                    onChange={handleChange}
                    disabled={isViewMode}
                    required
                    className={validationErrors.item_id ? "border-red-500" : ""}
                  />
                  {validationErrors.item_id && (
                    <p className="text-red-500 text-sm">
                      {validationErrors.item_id}
                    </p>
                  )}
                </div>

                <div className="space-y-2">
                  <Label htmlFor="quantity">Quantity</Label>
                  <Input
                    id="quantity"
                    name="quantity"
                    type="number"
                    min="1"
                    value={formData.quantity}
                    onChange={handleChange}
                    disabled={isViewMode}
                    required
                    className={
                      validationErrors.quantity ? "border-red-500" : ""
                    }
                  />
                  {validationErrors.quantity && (
                    <p className="text-red-500 text-sm">
                      {validationErrors.quantity}
                    </p>
                  )}
                </div>

                <div className="space-y-2">
                  <Label htmlFor="assigned_user_id">
                    Assigned User ID (UUID)
                  </Label>
                  <Input
                    id="assigned_user_id"
                    name="assigned_user_id"
                    value={formData.assigned_user_id}
                    onChange={handleChange}
                    disabled={isViewMode}
                    className={
                      validationErrors.assigned_user_id ? "border-red-500" : ""
                    }
                  />
                  {validationErrors.assigned_user_id && (
                    <p className="text-red-500 text-sm">
                      {validationErrors.assigned_user_id}
                    </p>
                  )}
                </div>

                <div className="space-y-2">
                  <Label htmlFor="supervisor_id">Supervisor ID (UUID)</Label>
                  <Input
                    id="supervisor_id"
                    name="supervisor_id"
                    value={formData.supervisor_id}
                    onChange={handleChange}
                    disabled={isViewMode}
                    className={
                      validationErrors.supervisor_id ? "border-red-500" : ""
                    }
                  />
                  {validationErrors.supervisor_id && (
                    <p className="text-red-500 text-sm">
                      {validationErrors.supervisor_id}
                    </p>
                  )}
                </div>

                <div className="space-y-2">
                  <Label htmlFor="customer_id">Customer ID (UUID)</Label>
                  <Input
                    id="customer_id"
                    name="customer_id"
                    value={formData.customer_id}
                    onChange={handleChange}
                    disabled={isViewMode}
                    className={
                      validationErrors.customer_id ? "border-red-500" : ""
                    }
                  />
                  {validationErrors.customer_id && (
                    <p className="text-red-500 text-sm">
                      {validationErrors.customer_id}
                    </p>
                  )}
                </div>

                {(isEditMode || isViewMode) && (
                  <div className="space-y-2">
                    <Label htmlFor="job_type">Job Type</Label>
                    <Input
                      id="job_type"
                      value={
                        formData.job_type.charAt(0).toUpperCase() +
                        formData.job_type.slice(1)
                      }
                      disabled
                    />
                  </div>
                )}

                {!isViewMode && isCreateMode && (
                  <div className="space-y-2">
                    <Label htmlFor="job_type">Job Type</Label>
                    <Select
                      value={formData.job_type}
                      onValueChange={(value) =>
                        handleSelectChange("job_type", value)
                      }
                      disabled={isViewMode || !isCreateMode}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="Select job type" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="manufacturing">
                          Manufacturing
                        </SelectItem>
                        <SelectItem value="qa">Quality Assurance</SelectItem>
                        <SelectItem value="service">Service</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                )}

                <div className="space-y-2">
                  <Label htmlFor="priority">Priority</Label>
                  {isViewMode ? (
                    <Input
                      id="priority"
                      value={
                        formData.priority.charAt(0).toUpperCase() +
                        formData.priority.slice(1)
                      }
                      disabled
                    />
                  ) : (
                    <Select
                      value={formData.priority}
                      onValueChange={(value) =>
                        handleSelectChange("priority", value)
                      }
                      disabled={isViewMode}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="Select priority" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="low">Low</SelectItem>
                        <SelectItem value="normal">Normal</SelectItem>
                        <SelectItem value="high">High</SelectItem>
                        <SelectItem value="urgent">Urgent</SelectItem>
                      </SelectContent>
                    </Select>
                  )}
                </div>

                <div className="space-y-2">
                  <Label htmlFor="status">Status</Label>
                  {isViewMode ? (
                    <Input
                      id="status"
                      value={
                        formData.status.charAt(0).toUpperCase() +
                        formData.status.replace("_", " ").slice(1)
                      }
                      disabled
                    />
                  ) : (
                    <Select
                      value={formData.status}
                      onValueChange={(value) =>
                        handleSelectChange("status", value)
                      }
                      disabled={isViewMode}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="Select status" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="pending">Pending</SelectItem>
                        <SelectItem value="in_progress">In Progress</SelectItem>
                        <SelectItem value="on_hold">On Hold</SelectItem>
                        <SelectItem value="completed">Completed</SelectItem>
                        <SelectItem value="cancelled">Cancelled</SelectItem>
                      </SelectContent>
                    </Select>
                  )}
                </div>

                <div className="space-y-2">
                  <Label htmlFor="due_date">Due Date</Label>
                  <Input
                    id="due_date"
                    name="due_date"
                    type="date"
                    value={formData.due_date}
                    onChange={handleChange}
                    disabled={isViewMode}
                  />
                </div>
              </div>
            </div>

            {/* Comments Section */}
            <div className="space-y-2">
              <Label htmlFor="comments">Comments</Label>
              <Textarea
                id="comments"
                name="comments"
                value={formData.comments}
                onChange={handleChange}
                disabled={isViewMode}
                rows={4}
              />
            </div>

            {/* Render job-type specific fields */}
            {formData.job_type === "manufacturing" && (
              <div>
                <h3 className="text-lg font-medium border-b pb-2 mb-4">
                  Manufacturing Details
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="production_line">Production Line</Label>
                    <Input
                      id="production_line"
                      name="production_line"
                      value={formData.production_line}
                      onChange={handleChange}
                      disabled={isViewMode}
                    />
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="batch_number">Batch Number</Label>
                    <Input
                      id="batch_number"
                      name="batch_number"
                      value={formData.batch_number}
                      onChange={handleChange}
                      disabled={isViewMode}
                    />
                  </div>

                  <div className="space-y-2 col-span-2">
                    <div className="flex items-center space-x-2">
                      <Switch
                        id="quality_check_required"
                        checked={formData.quality_check_required}
                        onCheckedChange={(checked) =>
                          handleSelectChange("quality_check_required", checked)
                        }
                        disabled={isViewMode}
                      />
                      <Label htmlFor="quality_check_required">
                        Quality Check Required
                      </Label>
                    </div>
                  </div>

                  <div className="space-y-2 col-span-2">
                    <Label htmlFor="raw_materials">Raw Materials (JSON)</Label>
                    <Textarea
                      id="raw_materials"
                      name="raw_materials"
                      value={formData.raw_materials}
                      onChange={handleChange}
                      disabled={isViewMode}
                      rows={4}
                    />
                  </div>
                </div>
              </div>
            )}

            {formData.job_type === "qa" && (
              <div>
                <h3 className="text-lg font-medium border-b pb-2 mb-4">
                  QA Details
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="test_procedure">Test Procedure</Label>
                    <Input
                      id="test_procedure"
                      name="test_procedure"
                      value={formData.test_procedure}
                      onChange={handleChange}
                      disabled={isViewMode}
                    />
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="inspection_type">Inspection Type</Label>
                    <Input
                      id="inspection_type"
                      name="inspection_type"
                      value={formData.inspection_type}
                      onChange={handleChange}
                      disabled={isViewMode}
                    />
                  </div>

                  <div className="space-y-2 col-span-2">
                    <Label htmlFor="pass_criteria">Pass Criteria (JSON)</Label>
                    <Textarea
                      id="pass_criteria"
                      name="pass_criteria"
                      value={formData.pass_criteria}
                      onChange={handleChange}
                      disabled={isViewMode}
                      rows={4}
                    />
                  </div>

                  <div className="space-y-2 col-span-2">
                    <Label htmlFor="qa_results">QA Results (JSON)</Label>
                    <Textarea
                      id="qa_results"
                      name="qa_results"
                      value={formData.qa_results}
                      onChange={handleChange}
                      disabled={isViewMode}
                      rows={4}
                    />
                  </div>
                </div>
              </div>
            )}

            {formData.job_type === "service" && (
              <div>
                <h3 className="text-lg font-medium border-b pb-2 mb-4">
                  Service Details
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="service_type">Service Type</Label>
                    <Input
                      id="service_type"
                      name="service_type"
                      value={formData.service_type}
                      onChange={handleChange}
                      disabled={isViewMode}
                    />
                  </div>

                  <div className="space-y-2 col-span-2">
                    <Label htmlFor="problem_description">
                      Problem Description
                    </Label>
                    <Textarea
                      id="problem_description"
                      name="problem_description"
                      value={formData.problem_description}
                      onChange={handleChange}
                      disabled={isViewMode}
                      rows={3}
                    />
                  </div>

                  <div className="space-y-2 col-span-2">
                    <Label htmlFor="diagnosis">Diagnosis</Label>
                    <Textarea
                      id="diagnosis"
                      name="diagnosis"
                      value={formData.diagnosis}
                      onChange={handleChange}
                      disabled={isViewMode}
                      rows={3}
                    />
                  </div>

                  <div className="space-y-2 col-span-2">
                    <Label htmlFor="solution">Solution</Label>
                    <Textarea
                      id="solution"
                      name="solution"
                      value={formData.solution}
                      onChange={handleChange}
                      disabled={isViewMode}
                      rows={3}
                    />
                  </div>

                  <div className="space-y-2 col-span-2">
                    <Label htmlFor="parts_replaced">
                      Parts Replaced (JSON)
                    </Label>
                    <Textarea
                      id="parts_replaced"
                      name="parts_replaced"
                      value={formData.parts_replaced}
                      onChange={handleChange}
                      disabled={isViewMode}
                      rows={4}
                    />
                  </div>
                </div>
              </div>
            )}

            {/* Job History Section (view mode only) */}
            {isViewMode && job && history && history.length > 0 && (
              <div>
                <h3 className="text-lg font-medium border-b pb-2 mb-4">
                  Job History
                </h3>
                <div className="space-y-4">
                  {history.map((entry) => (
                    <div
                      key={entry.id}
                      className="border rounded-md p-3 bg-gray-50"
                    >
                      <div className="flex justify-between items-start">
                        <div>
                          <p className="font-semibold capitalize">
                            {entry.action.replace("_", " ")}
                          </p>
                          {entry.previous_status && entry.new_status && (
                            <p className="text-sm text-gray-600">
                              Status changed from{" "}
                              <span className="font-medium">
                                {entry.previous_status.toUpperCase()}
                              </span>{" "}
                              to{" "}
                              <span className="font-medium">
                                {entry.new_status.toUpperCase()}
                              </span>
                            </p>
                          )}
                          {entry.notes && (
                            <p className="text-sm mt-1">{entry.notes}</p>
                          )}
                        </div>
                        <div className="text-right">
                          {entry.created_at && (
                            <p className="text-xs text-gray-500 flex items-center">
                              <Clock className="h-3 w-3 mr-1" />
                              {formatDate(entry.created_at)}
                            </p>
                          )}
                          {entry.user_id && (
                            <p className="text-xs text-gray-500 mt-1">
                              By: {entry.user_id.substring(0, 8)}...
                            </p>
                          )}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </CardContent>

          <CardFooter className="flex justify-between">
            <Button type="button" variant="outline" onClick={onBack}>
              Cancel
            </Button>

            {!isViewMode && (
              <Button type="submit" disabled={loading}>
                {loading ? (
                  <>
                    <div className="animate-spin h-4 w-4 border-2 border-current border-t-transparent rounded-full mr-2"></div>
                    Saving...
                  </>
                ) : isEditMode ? (
                  "Update Job"
                ) : (
                  "Create Job"
                )}
              </Button>
            )}
          </CardFooter>
        </Card>
      </form>
    </div>
  );
};

export default JobFormView;
