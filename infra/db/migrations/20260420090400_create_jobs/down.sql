DROP TRIGGER IF EXISTS log_job_status_change_trigger ON public.jobs;
DROP FUNCTION IF EXISTS public.log_job_status_change();

DROP TABLE IF EXISTS public.job_history CASCADE;
DROP TABLE IF EXISTS public.service_job CASCADE;
DROP TABLE IF EXISTS public.qa_job CASCADE;
DROP TABLE IF EXISTS public.manufacturing_job CASCADE;
DROP TABLE IF EXISTS public.jobs CASCADE;
