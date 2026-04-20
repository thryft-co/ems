import React, { useState, useEffect } from "react";
import {
  deleteJob,
  getServiceJobs,
} from "@/features/jobs/services/jobService";
import { Job } from "@/features/jobs/types/job";
import DeleteConfirmationDialog from "@/shared/components/DeleteConfirmationDialog";
import { Button } from "@/shared/ui/button";
import JobCard from "./JobCard.tsx";
import JobFormView from "./JobFormView";
import { PlusIcon, RefreshCw } from "lucide-react";

const ServiceJobList: React.FC = () => {
  const [jobs, setJobs] = useState<Job[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null);
  const [showForm, setShowForm] = useState<boolean>(false);
  const [formMode, setFormMode] = useState<"view" | "edit" | "create">(
    "create",
  );
  const [editJobId, setEditJobId] = useState<string | null>(null);
  const [deleteJobId, setDeleteJobId] = useState<string | null>(null);
  const [deleteJobNumber, setDeleteJobNumber] = useState<string>("");
  const [showDeleteDialog, setShowDeleteDialog] = useState<boolean>(false);

  const fetchJobs = async (): Promise<void> => {
    try {
      setLoading(true);
      setError(null);
      const data = await getServiceJobs();
      console.log("Fetched jobs:", data);
      setJobs(data);
    } catch (err) {
      console.error("Failed to fetch Service jobs:", err);
      setError("Failed to load Service jobs. Please try again.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchJobs();
  }, []);

  const handleViewJob = (jobId: string): void => {
    console.log(`View job clicked with ID: ${jobId} (type: ${typeof jobId})`);
    if (!jobId) {
      console.error("Invalid job ID for view:", jobId);
      return;
    }
    setSelectedJobId(jobId);
    setFormMode("view");
    setShowForm(true);
  };

  const handleEditJob = (jobId: string): void => {
    console.log(`Edit job clicked with ID: ${jobId} (type: ${typeof jobId})`);
    if (!jobId) {
      console.error("Invalid job ID for edit:", jobId);
      return;
    }
    setEditJobId(jobId);
    setFormMode("edit");
    setShowForm(true);
  };

  const handleDeleteJob = (jobId: string, jobNumber: string): void => {
    console.log(
      `Delete job clicked with ID: ${jobId} (type: ${typeof jobId}), number: ${jobNumber}`,
    );
    if (!jobId) {
      console.error("Invalid job ID for delete:", jobId);
      return;
    }
    setDeleteJobId(jobId);
    setDeleteJobNumber(jobNumber);
    setShowDeleteDialog(true);
  };

  const handleAddJob = (): void => {
    setFormMode("create");
    setShowForm(true);
    setSelectedJobId(null);
    setEditJobId(null);
  };

  const handleBackFromForm = (): void => {
    // Check if we're coming from view mode and need to switch to edit mode
    if (formMode === "view" && selectedJobId) {
      // Switch to edit mode for the same job
      setEditJobId(selectedJobId);
      setSelectedJobId(null);
      setFormMode("edit");
      return;
    }

    // Otherwise, just go back to the list
    setShowForm(false);
    setSelectedJobId(null);
    setEditJobId(null);
    setFormMode("create"); // Reset to default
  };

  const handleFormSave = (): void => {
    fetchJobs();
    setShowForm(false);
    setSelectedJobId(null);
    setEditJobId(null);
    setFormMode("create"); // Reset to default
  };

  const handleJobDeleted = (): void => {
    console.log("Job successfully deleted, refreshing list");
    fetchJobs();
  };

  const handleCloseDeleteDialog = (): void => {
    setShowDeleteDialog(false);
    setDeleteJobId(null);
    setDeleteJobNumber("");
  };

  // If showing form for view, edit, or create
  if (showForm) {
    // Determine which job ID to use based on mode
    const jobId =
      formMode === "edit"
        ? editJobId
        : formMode === "view"
          ? selectedJobId
          : undefined;

    console.log(
      `Showing form in ${formMode} mode for job ID: ${jobId || "new"}`,
    );

    return (
      <JobFormView
        jobId={jobId || undefined}
        jobType="service"
        mode={formMode}
        onSave={handleFormSave}
        onBack={handleBackFromForm}
      />
    );
  }

  return (
    <div className="w-full">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-xl font-semibold">Service Jobs</h2>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={fetchJobs}
            disabled={loading}
          >
            <RefreshCw
              className={`h-4 w-4 mr-2 ${loading ? "animate-spin" : ""}`}
            />
            Refresh
          </Button>
          <Button size="sm" onClick={handleAddJob}>
            <PlusIcon className="h-4 w-4 mr-2" />
            New Job
          </Button>
        </div>
      </div>

      {error && (
        <div
          className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4"
          role="alert"
        >
          <p>{error}</p>
        </div>
      )}

      {loading ? (
        <div className="text-center p-8">
          <div className="animate-spin h-8 w-8 border-2 border-primary border-t-transparent rounded-full mx-auto"></div>
          <p className="mt-2 text-muted-foreground">Loading jobs...</p>
        </div>
      ) : jobs.length === 0 ? (
        <div className="text-center p-8 border rounded-lg">
          <p className="text-muted-foreground">No service jobs found.</p>
          <Button variant="outline" className="mt-4" onClick={handleAddJob}>
            Create Your First Job
          </Button>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
          {jobs.map((job) => (
            <JobCard
              key={job.id}
              job={job}
              onView={() => handleViewJob(job.id)}
              onEdit={() => handleEditJob(job.id)}
              onDelete={() => handleDeleteJob(job.id, job.job_number)}
            />
          ))}
        </div>
      )}

      <DeleteConfirmationDialog
        entityLabel="job"
        isOpen={showDeleteDialog}
        onClose={handleCloseDeleteDialog}
        itemLabel={deleteJobNumber || deleteJobId || undefined}
        onConfirm={() => deleteJob(deleteJobId || "")}
        onDeleted={handleJobDeleted}
      />
    </div>
  );
};

export default ServiceJobList;
