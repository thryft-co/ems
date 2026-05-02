import React, { useState, useEffect } from "react";
import { deleteJob, getManufacturingJobs } from "@/features/jobs/services/jobService";
import { Job } from "@/features/jobs/types/job";
import DeleteConfirmationDialog from "@/shared/components/DeleteConfirmationDialog";
import { Button } from "@/shared/ui/button";
import JobCard from "./JobCard.tsx";
import JobFormView from "./JobFormView";
import { PlusIcon, RefreshCw, Factory } from "lucide-react";

const ManufacturingJobList: React.FC = () => {
  const [jobs, setJobs] = useState<Job[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null);
  const [showForm, setShowForm] = useState<boolean>(false);
  const [formMode, setFormMode] = useState<"view" | "edit" | "create">("create");
  const [editJobId, setEditJobId] = useState<string | null>(null);
  const [deleteJobId, setDeleteJobId] = useState<string | null>(null);
  const [deleteJobNumber, setDeleteJobNumber] = useState<string>("");
  const [showDeleteDialog, setShowDeleteDialog] = useState<boolean>(false);

  const fetchJobs = async (): Promise<void> => {
    try {
      setLoading(true);
      setError(null);
      const data = await getManufacturingJobs();
      setJobs(data);
    } catch (err) {
      console.error("Failed to fetch manufacturing jobs:", err);
      setError("Failed to load manufacturing jobs. Please try again.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { fetchJobs(); }, []);

  const handleViewJob = (jobId: string): void => {
    if (!jobId) return;
    setSelectedJobId(jobId);
    setFormMode("view");
    setShowForm(true);
  };

  const handleEditJob = (jobId: string): void => {
    if (!jobId) return;
    setEditJobId(jobId);
    setFormMode("edit");
    setShowForm(true);
  };

  const handleDeleteJob = (jobId: string, jobNumber: string): void => {
    if (!jobId) return;
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
    if (formMode === "view" && selectedJobId) {
      setEditJobId(selectedJobId);
      setSelectedJobId(null);
      setFormMode("edit");
      return;
    }
    setShowForm(false);
    setSelectedJobId(null);
    setEditJobId(null);
    setFormMode("create");
  };

  const handleFormSave = (): void => {
    fetchJobs();
    setShowForm(false);
    setSelectedJobId(null);
    setEditJobId(null);
    setFormMode("create");
  };

  const handleJobDeleted = (): void => { fetchJobs(); };
  const handleCloseDeleteDialog = (): void => {
    setShowDeleteDialog(false);
    setDeleteJobId(null);
    setDeleteJobNumber("");
  };

  if (showForm) {
    const jobId = formMode === "edit" ? editJobId : formMode === "view" ? selectedJobId : undefined;
    return <JobFormView jobId={jobId || undefined} jobType="manufacturing" mode={formMode} onSave={handleFormSave} onBack={handleBackFromForm} />;
  }

  return (
    <div className="w-full">
      {/* Header */}
      <div className="flex items-center justify-between mb-5">
        <h2 className="text-[17px] font-semibold text-foreground">Manufacturing Jobs</h2>
        <div className="flex gap-2">
          <Button variant="ghost" size="sm" onClick={fetchJobs} disabled={loading} className="h-9 text-[13px]">
            <RefreshCw className={`h-3.5 w-3.5 mr-1.5 ${loading ? "animate-spin" : ""}`} />
            <span className="hidden sm:inline">Refresh</span>
          </Button>
          <Button size="sm" onClick={handleAddJob} className="h-9 text-[13px]">
            <PlusIcon className="h-3.5 w-3.5 mr-1.5" />New Job
          </Button>
        </div>
      </div>

      {/* Error */}
      {error && (
        <div className="text-[13px] text-destructive bg-destructive/8 rounded-xl px-4 py-3 mb-4">{error}</div>
      )}

      {/* Content */}
      {loading ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {[1, 2, 3].map((i) => (
            <div key={i} className="bg-card rounded-2xl border-[0.5px] border-border/60 p-4 space-y-3 animate-pulse">
              <div className="flex items-center gap-2.5">
                <div className="w-8 h-8 rounded-[8px] skeleton" />
                <div className="space-y-1.5 flex-1">
                  <div className="h-4 w-24 skeleton" />
                  <div className="h-3 w-16 skeleton" />
                </div>
              </div>
              <div className="grid grid-cols-2 gap-2">
                <div className="h-3 skeleton" />
                <div className="h-3 skeleton" />
              </div>
            </div>
          ))}
        </div>
      ) : jobs.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <div className="w-12 h-12 rounded-2xl bg-secondary/60 flex items-center justify-center mb-4">
            <Factory className="w-6 h-6 text-muted-foreground/50" />
          </div>
          <h3 className="text-[17px] font-semibold text-foreground mb-1">No Jobs Yet</h3>
          <p className="text-[13px] text-muted-foreground mb-5 max-w-[240px]">Create your first manufacturing job to get started.</p>
          <Button onClick={handleAddJob}>
            <PlusIcon className="h-4 w-4 mr-2" />Create Your First Job
          </Button>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {jobs.map((job) => (
            <JobCard key={job.id} job={job} onView={() => handleViewJob(job.id)} onEdit={() => handleEditJob(job.id)} onDelete={() => handleDeleteJob(job.id, job.job_number)} />
          ))}
        </div>
      )}

      <DeleteConfirmationDialog entityLabel="job" isOpen={showDeleteDialog} onClose={handleCloseDeleteDialog} itemLabel={deleteJobNumber || deleteJobId || undefined} onConfirm={() => deleteJob(deleteJobId || "")} onDeleted={handleJobDeleted} />
    </div>
  );
};

export default ManufacturingJobList;
