# Spring Boot 实战篇 - 第13-19章
# Spring Boot Practice - Chapters 13-19

> 文件处理、定时任务、邮件、CORS、统一响应、分页、导出
> File Handling, Scheduled Tasks, Email, CORS, Unified Response, Pagination, Export

---

## 目录 / Table of Contents

1. [第13章：文件上传与下载](#第13章文件上传与下载)
2. [第14章：定时任务与异步执行](#第14章定时任务与异步执行)
3. [第15章：邮件发送、通知推送](#第15章邮件发送通知推送)
4. [第16章：跨域请求处理](#第16章跨域请求处理)
5. [第17章：前后端接口统一响应结构](#第17章前后端接口统一响应结构)
6. [第18章：分页查询接口标准实现](#第18章分页查询接口标准实现)
7. [第19章：文件导出与报表生成](#第19章文件导出与报表生成)

---

## 第13章：文件上传与下载

### 文件上传对比 / File Upload Comparison

#### Spring Boot - 文件上传

```java
// 1. 配置文件上传
// application.yml
spring:
  servlet:
    multipart:
      enabled: true
      max-file-size: 10MB
      max-request-size: 10MB
      file-size-threshold: 2MB

// 2. 文件上传控制器
@RestController
@RequestMapping("/api/files")
public class FileController {

    @Value("${app.upload.path:${user.home}/uploads}")
    private String uploadPath;

    // 单文件上传
    @PostMapping("/upload")
    public Result<FileUploadResponse> uploadFile(
        @RequestParam("file") MultipartFile file
    ) {
        // 验证文件
        if (file.isEmpty()) {
            return Result.error("文件不能为空");
        }

        // 验证文件类型
        String contentType = file.getContentType();
        if (!isAllowedContentType(contentType)) {
            return Result.error("不支持的文件类型");
        }

        // 验证文件大小
        if (file.getSize() > 10 * 1024 * 1024) {
            return Result.error("文件大小超过限制");
        }

        try {
            // 生成唯一文件名
            String originalFilename = file.getOriginalFilename();
            String extension = getFileExtension(originalFilename);
            String filename = UUID.randomUUID() + "." + extension;

            // 保存文件
            Path path = Paths.get(uploadPath, filename);
            Files.createDirectories(path.getParent());
            file.transferTo(path.toFile());

            // 返回文件信息
            FileUploadResponse response = FileUploadResponse.builder()
                .filename(filename)
                .originalName(originalFilename)
                .size(file.getSize())
                .contentType(contentType)
                .url("/api/files/download/" + filename)
                .build();

            return Result.success(response);

        } catch (IOException e) {
            return Result.error("文件上传失败");
        }
    }

    // 多文件上传
    @PostMapping("/upload/multiple")
    public Result<List<FileUploadResponse>> uploadFiles(
        @RequestParam("files") MultipartFile[] files
    ) {
        List<FileUploadResponse> responses = new ArrayList<>();

        for (MultipartFile file : files) {
            if (!file.isEmpty()) {
                try {
                    String filename = saveFile(file);
                    responses.add(FileUploadResponse.builder()
                        .filename(filename)
                        .originalName(file.getOriginalFilename())
                        .size(file.getSize())
                        .build());
                } catch (IOException e) {
                    // 继续处理其他文件
                }
            }
        }

        return Result.success(responses);
    }

    // 文件下载
    @GetMapping("/download/{filename:.+}")
    public ResponseEntity<Resource> downloadFile(@PathVariable String filename) {
        try {
            Path path = Paths.get(uploadPath, filename);
            Resource resource = new UrlResource(path.toUri());

            if (resource.exists() && resource.isReadable()) {
                String contentType = Files.probeContentType(path);
                if (contentType == null) {
                    contentType = "application/octet-stream";
                }

                return ResponseEntity.ok()
                    .contentType(MediaType.parseMediaType(contentType))
                    .header(HttpHeaders.CONTENT_DISPOSITION,
                        "attachment; filename=\"" + filename + "\"")
                    .body(resource);
            } else {
                return ResponseEntity.notFound().build();
            }
        } catch (Exception e) {
            return ResponseEntity.internalServerError().build();
        }
    }

    // 显示图片
    @GetMapping("/image/{filename:.+}")
    public ResponseEntity<Resource> serveImage(@PathVariable String filename) {
        try {
            Path path = Paths.get(uploadPath, filename);
            Resource resource = new UrlResource(path.toUri());

            if (resource.exists() && resource.isReadable()) {
                return ResponseEntity.ok()
                    .header(HttpHeaders.CONTENT_TYPE, "image/jpeg")
                    .body(resource);
            } else {
                return ResponseEntity.notFound().build();
            }
        } catch (Exception e) {
            return ResponseEntity.internalServerError().build();
        }
    }

    private boolean isAllowedContentType(String contentType) {
        List<String> allowed = Arrays.asList(
            "image/jpeg", "image/png", "image/gif",
            "application/pdf",
            "text/plain"
        );
        return allowed.contains(contentType);
    }
}
```

#### Nexus - 文件上传

```rust
use hiver::prelude::*;
use hiver_macros::{controller, post, get};
use multer::{Multipart, Field};
use std::path::PathBuf;
use uuid::Uuid;

#[controller]
struct FileController;

// 文件上传响应
#[derive(Serialize)]
pub struct FileUploadResponse {
    pub filename: String,
    pub original_name: String,
    pub size: u64,
    pub content_type: String,
    pub url: String,
}

// 允许的文件类型
const ALLOWED_CONTENT_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "application/pdf",
    "text/plain",
];

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB

/// 单文件上传
#[post("/api/files/upload")]
async fn upload_file(
    mut req: Request,
    #[state] config: Arc<UploadConfig>,
) -> Result<Json<FileUploadResponse>, Error> {
    // 解析 multipart 表单
    let boundary = req.get_boundary()
        .ok_or_else(|| Error::bad_request("缺少 boundary"))?;

    let form = Multipart::new(&mut req, boundary);

    // 遍历字段
    while let Some(field) = form.next_field().await.map_err(|e| Error::bad_request(&e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            // 获取文件名
            let original_name = field.file_name()
                .ok_or_else(|| Error::bad_request("文件名不能为空"))?
                .to_string();

            // 获取文件类型
            let content_type = field.content_type()
                .unwrap_or("application/octet-stream")
                .to_string();

            // 验证文件类型
            if !ALLOWED_CONTENT_TYPES.contains(&content_type.as_str()) {
                return Err(Error::bad_request("不支持的文件类型"));
            }

            // 读取文件数据
            let data = field.bytes().await.map_err(|e| Error::bad_request(&e.to_string()))?;

            // 验证文件大小
            if data.len() > MAX_FILE_SIZE {
                return Err(Error::bad_request("文件大小超过限制"));
            }

            // 生成唯一文件名
            let extension = PathBuf::from(&original_name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("bin");
            let filename = format!("{}.{}", Uuid::new_v4(), extension);

            // 保存文件
            let path = config.upload_path.join(&filename);
            tokio::fs::create_dir_all(&path.parent().unwrap()).await
                .map_err(|e| Error::internal(&e.to_string()))?;
            tokio::fs::write(&path, data).await
                .map_err(|e| Error::internal(&e.to_string()))?;

            // 返回响应
            let response = FileUploadResponse {
                filename: filename.clone(),
                original_name,
                size: data.len() as u64,
                content_type,
                url: format!("/api/files/download/{}", filename),
            };

            return Ok(Json(response));
        }
    }

    Err(Error::bad_request("未找到文件"))
}

/// 多文件上传
#[post("/api/files/upload/multiple")]
async fn upload_files(
    mut req: Request,
    #[state] config: Arc<UploadConfig>,
) -> Result<Json<Vec<FileUploadResponse>>, Error> {
    let boundary = req.get_boundary()
        .ok_or_else(|| Error::bad_request("缺少 boundary"))?;

    let form = Multipart::new(&mut req, boundary);
    let mut responses = Vec::new();

    while let Some(field) = form.next_field().await.map_err(|e| Error::bad_request(&e.to_string()))? {
        if let Some(original_name) = field.file_name() {
            if !original_name.is_empty() {
                let content_type = field.content_type().unwrap_or("application/octet-stream");
                let data = field.bytes().await.map_err(|e| Error::bad_request(&e.to_string()))?;

                if data.len() <= MAX_FILE_SIZE {
                    let extension = PathBuf::from(original_name)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("bin");
                    let filename = format!("{}.{}", Uuid::new_v4(), extension);

                    let path = config.upload_path.join(&filename);
                    if let Ok(()) = tokio::fs::write(&path, data).await {
                        responses.push(FileUploadResponse {
                            filename: filename.clone(),
                            original_name: original_name.to_string(),
                            size: data.len() as u64,
                            content_type: content_type.to_string(),
                            url: format!("/api/files/download/{}", filename),
                        });
                    }
                }
            }
        }
    }

    Ok(Json(responses))
}

/// 文件下载
#[get("/api/files/download/:filename")]
async fn download_file(
    filename: String,
    #[state] config: Arc<UploadConfig>,
) -> Result<Response, Error> {
    let path = config.upload_path.join(&filename);

    // 检查文件是否存在
    if !path.exists() {
        return Err(Error::not_found("File", &filename));
    }

    // 读取文件
    let data = tokio::fs::read(&path).await
        .map_err(|e| Error::internal(&e.to_string()))?;

    // 确定内容类型
    let content_type = mime_guess::from_path(&filename)
        .first_or_octet_stream()
        .to_string();

    // 构建响应
    let response = Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .header("Content-Disposition", format!("attachment; filename=\"{}\"", filename))
        .header("Content-Length", data.len())
        .body(Body::from(data))
        .map_err(|e| Error::internal(&e.to_string()))?;

    Ok(response)
}

/// 显示图片
#[get("/api/files/image/:filename")]
async fn serve_image(
    filename: String,
    #[state] config: Arc<UploadConfig>,
) -> Result<Response, Error> {
    let path = config.upload_path.join(&filename);

    if !path.exists() {
        return Err(Error::not_found("Image", &filename));
    }

    let data = tokio::fs::read(&path).await
        .map_err(|e| Error::internal(&e.to_string()))?;

    let content_type = mime_guess::from_path(&filename)
        .first_or_octet_stream()
        .to_string();

    let response = Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .header("Cache-Control", "public, max-age=3600")
        .body(Body::from(data))
        .map_err(|e| Error::internal(&e.to_string()))?;

    Ok(response)
}
```

---

## 第14章：定时任务与异步执行

### 定时任务对比 / Scheduled Tasks Comparison

#### Spring Boot - @Scheduled

```java
// 1. 启用定时任务
@Configuration
@EnableScheduling
public class SchedulingConfig {
}

// 2. 定时任务
@Component
@Slf4j
public class ScheduledTasks {

    @Autowired
    private UserService userService;

    @Autowired
    private EmailService emailService;

    // 固定延迟执行
    @Scheduled(fixedDelay = 5000)  // 上次执行完成后5秒再执行
    public void fixedDelayTask() {
        log.info("Fixed delay task executed");
    }

    // 固定频率执行
    @Scheduled(fixedRate = 10000)  // 每10秒执行一次
    public void fixedRateTask() {
        log.info("Fixed rate task executed");
    }

    // Cron 表达式
    @Scheduled(cron = "0 0 * * * ?")  // 每小时整点执行
    public void hourlyTask() {
        log.info("Hourly task executed");
    }

    // Cron: 每天凌晨2点执行
    @Scheduled(cron = "0 0 2 * * ?")
    public void dailyCleanup() {
        log.info("Starting daily cleanup");
        // 清理过期数据
        userService.cleanupExpiredUsers();
    }

    // Cron: 每周一早上10点执行
    @Scheduled(cron = "0 0 10 ? * MON")
    public void weeklyReport() {
        log.info("Generating weekly report");
        // 生成周报
    }

    // 支持参数化
    @Scheduled(cron = "${s cleanup.cron:0 0 2 * * ?}")
    public void parametrizedCleanup() {
        // ...
    }

    // 带异常处理
    @Scheduled(fixedRate = 60000)
    public void taskWithExceptionHandling() {
        try {
            // 任务逻辑
        } catch (Exception e) {
            log.error("Task failed", e);
        }
    }
}

// 3. 异步任务执行
@Configuration
@EnableAsync
public class AsyncConfig implements AsyncConfigurer {

    @Override
    public Executor getAsyncExecutor() {
        ThreadPoolTaskExecutor executor = new ThreadPoolTaskExecutor();
        executor.setCorePoolSize(5);
        executor.setMaxPoolSize(10);
        executor.setQueueCapacity(100);
        executor.setThreadNamePrefix("async-");
        executor.initialize();
        return executor;
    }

    @Override
    public AsyncUncaughtExceptionHandler getAsyncUncaughtExceptionHandler() {
        return (throwable, method, params) ->
            log.error("Async method {} threw exception", method, throwable);
    }
}

@Component
@Slf4j
public class AsyncService {

    @Autowired
    private EmailService emailService;

    // 异步发送邮件
    @Async
    public CompletableFuture<Void> sendEmailAsync(String to, String subject, String body) {
        try {
            emailService.send(to, subject, body);
            log.info("Email sent to {}", to);
            return CompletableFuture.completedFuture(null);
        } catch (Exception e) {
            log.error("Failed to send email", e);
            throw new CompletionException(e);
        }
    }

    // 批量处理
    @Async
    public CompletableFuture<List<User>> processUsersAsync(List<Long> userIds) {
        List<User> users = new ArrayList<>();
        for (Long id : userIds) {
            // 处理用户
            users.add(userService.findById(id));
        }
        return CompletableFuture.completedFuture(users);
    }
}

// 4. 使用异步服务
@RestController
public class NotificationController {

    @Autowired
    private AsyncService asyncService;

    @PostMapping("/notify")
    public Result<String> sendNotification(@RequestBody NotificationRequest request) {
        // 异步发送，立即返回
        asyncService.sendEmailAsync(request.getTo(), request.getSubject(), request.getBody());
        return Result.success("Notification queued");
    }
}
```

#### Nexus - 定时任务与异步

```rust
use hiver_runtime::scheduler::{Scheduler, TaskSchedule};
use hiver_macros::scheduled;
use std::sync::Arc;
use tokio::task::JoinHandle;

// 1. 定时任务配置
#[service]
pub struct ScheduledTasksService {
    #[autowired]
    user_service: Arc<UserService>,

    #[autowired]
    email_service: Arc<EmailService>,
}

impl ScheduledTasksService {
    // 固定延迟执行
    #[scheduled(fixed_delay_ms = 5000)]
    pub async fn fixed_delay_task(&self) {
        log::info!("Fixed delay task executed");
    }

    // 固定频率执行
    #[scheduled(fixed_rate_ms = 10000)]
    pub async fn fixed_rate_task(&self) {
        log::info!("Fixed rate task executed");
    }

    // Cron 表达式: 每小时整点
    #[scheduled(cron = "0 0 * * * *")]
    pub async fn hourly_task(&self) {
        log::info!("Hourly task executed");
    }

    // Cron: 每天凌晨2点
    #[scheduled(cron = "0 0 2 * * *")]
    pub async fn daily_cleanup(&self) {
        log::info!("Starting daily cleanup");
        self.user_service.cleanup_expired_users().await;
    }

    // Cron: 每周一早上10点
    #[scheduled(cron = "0 0 10 * * 1")]
    pub async fn weekly_report(&self) {
        log::info!("Generating weekly report");
    }

    // 带异常处理
    #[scheduled(fixed_rate_ms = 60000)]
    pub async fn task_with_exception_handling(&self) {
        if let Err(e) = self.do_task().await {
            log::error!("Task failed: {}", e);
        }
    }

    async fn do_task(&self) -> Result<(), Error> {
        // 任务逻辑
        Ok(())
    }
}

// 2. 手动配置调度器
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut scheduler = Scheduler::new();

    // 添加固定延迟任务
    scheduler.schedule_fixed_delay(
        Duration::from_secs(5),
        || async {
            log::info!("Fixed delay task");
        }
    ).await;

    // 添加固定频率任务
    scheduler.schedule_fixed_rate(
        Duration::from_secs(10),
        || async {
            log::info!("Fixed rate task");
        }
    ).await;

    // 添加 Cron 任务
    scheduler.schedule_cron(
        "0 0 * * * *",  // 每小时
        || async {
            log::info!("Cron task");
        }
    ).await?;

    // 启动调度器
    scheduler.start().await?;

    Ok(())
}

// 3. 异步任务执行
#[service]
pub struct AsyncService {
    #[autowired]
    email_service: Arc<EmailService>,

    #[autowired]
    user_service: Arc<UserService>,
}

impl AsyncService {
    // 异步发送邮件
    pub async fn send_email_async(
        &self,
        to: String,
        subject: String,
        body: String,
    ) -> JoinHandle<Result<(), EmailError>> {
        let email_service = self.email_service.clone();
        tokio::spawn(async move {
            email_service.send(&to, &subject, &body).await?;
            log::info!("Email sent to {}", to);
            Ok(())
        })
    }

    // 批量处理
    pub async fn process_users_async(
        &self,
        user_ids: Vec<i64>,
    ) -> JoinHandle<Vec<User>> {
        let user_service = self.user_service.clone();
        tokio::spawn(async move {
            let mut users = Vec::new();
            for id in user_ids {
                if let Some(user) = user_service.find_by_id(id).await {
                    users.push(user);
                }
            }
            users
        })
    }
}

// 4. 使用异步服务
#[controller]
struct NotificationController;

#[post("/api/notify")]
async fn send_notification(
    #[request_body] request: NotificationRequest,
    #[state] async_service: Arc<AsyncService>,
) -> Json<&'static str> {
    // 异步发送，立即返回
    async_service.send_email_async(
        request.to,
        request.subject,
        request.body,
    );

    Json("Notification queued")
}

// 5. 任务状态监控
#[service]
pub class TaskMonitorService {
    active_tasks: Arc<RwLock<HashMap<String, TaskInfo>>>,
}

pub struct TaskInfo {
    pub id: String,
    pub status: TaskStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub enum TaskStatus {
    Running,
    Completed,
    Failed(String),
}
```

---

## 第15章：邮件发送、通知推送

### 邮件服务对比 / Email Service Comparison

#### Spring Boot - 邮件发送

```java
// 1. 配置
// application.yml
spring:
  mail:
    host: smtp.example.com
    port: 587
    username: ${EMAIL_USERNAME}
    password: ${EMAIL_PASSWORD}
    protocol: smtp
    properties:
      mail:
        smtp:
          auth: true
          starttls:
            enable: true
          connectiontimeout: 5000
          timeout: 5000
          writetimeout: 5000

// 2. 邮件服务
@Service
@Slf4j
public class EmailService {

    @Autowired
    private JavaMailSender mailSender;

    @Autowired
    private TemplateEngine templateEngine;

    // 发送简单文本邮件
    public void sendSimpleEmail(String to, String subject, String text) {
        try {
            SimpleMailMessage message = new SimpleMailMessage();
            message.setTo(to);
            message.setSubject(subject);
            message.setText(text);
            message.setFrom("noreply@example.com");

            mailSender.send(message);
            log.info("Email sent to {}", to);
        } catch (MailException e) {
            log.error("Failed to send email to {}", to, e);
            throw new EmailException("Failed to send email", e);
        }
    }

    // 发送 HTML 邮件
    public void sendHtmlEmail(String to, String subject, String htmlContent) {
        try {
            MimeMessage message = mailSender.createMimeMessage();
            MimeMessageHelper helper = new MimeMessageHelper(message, true, "UTF-8");

            helper.setTo(to);
            helper.setSubject(subject);
            helper.setText(htmlContent, true);
            helper.setFrom("noreply@example.com");

            mailSender.send(message);
            log.info("HTML email sent to {}", to);
        } catch (MessagingException e) {
            log.error("Failed to send HTML email to {}", to, e);
            throw new EmailException("Failed to send HTML email", e);
        }
    }

    // 使用模板发送邮件
    public void sendTemplateEmail(String to, String subject, String templateName, Map<String, Object> variables) {
        try {
            Context context = new Context();
            context.setVariables(variables);

            String htmlContent = templateEngine.process(templateName, context);

            sendHtmlEmail(to, subject, htmlContent);
        } catch (Exception e) {
            log.error("Failed to send template email to {}", to, e);
            throw new EmailException("Failed to send template email", e);
        }
    }

    // 发送带附件的邮件
    public void sendEmailWithAttachment(
        String to,
        String subject,
        String text,
        String attachmentPath
    ) {
        try {
            MimeMessage message = mailSender.createMimeMessage();
            MimeMessageHelper helper = new MimeMessageHelper(message, true);

            helper.setTo(to);
            helper.setSubject(subject);
            helper.setText(text, true);
            helper.setFrom("noreply@example.com");

            // 添加附件
            FileSystemResource file = new FileSystemResource(new File(attachmentPath));
            helper.addAttachment(file.getFilename(), file);

            mailSender.send(message);
            log.info("Email with attachment sent to {}", to);
        } catch (MessagingException e) {
            log.error("Failed to send email with attachment to {}", to, e);
            throw new EmailException("Failed to send email with attachment", e);
        }
    }

    // 批量发送
    @Async
    public void sendBulkEmails(List<String> recipients, String subject, String content) {
        for (String to : recipients) {
            try {
                sendSimpleEmail(to, subject, content);
            } catch (Exception e) {
                log.error("Failed to send email to {}", to, e);
            }
        }
    }
}
```

#### Nexus - 邮件服务

```rust
use lettre::{
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
    message::{header, SinglePart},
};
use handlebars::Handlebars;
use std::sync::Arc;

#[service]
pub struct EmailService {
    smtp_transport: Arc<SmashSyncTransport<SmtpTransport>>,

    #[config(prefix = "email.smtp")]
    smtp_config: SmtpConfig,

    #[autowired]
    template_engine: Arc<TemplateEngine>,
}

#[config(prefix = "email.smtp")]
pub struct SmtpConfig {
    pub host: String,
    #[config(default = "587")]
    pub port: u16,
    pub username: String,
    pub password: String,
    #[config(default = "noreply@example.com")]
    pub from: String,
}

impl EmailService {
    // 发送简单文本邮件
    pub async fn send(
        &self,
        to: &str,
        subject: &str,
        text: &str,
    ) -> Result<(), EmailError> {
        let email = Message::builder()
            .from(self.smtp_config.from.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .singlepart(SinglePart::plain().body(text.to_string()))?;

        self.transport.send(&email).await?;
        log::info!("Email sent to {}", to);
        Ok(())
    }

    // 发送 HTML 邮件
    pub async fn send_html(
        &self,
        to: &str,
        subject: &str,
        html: &str,
    ) -> Result<(), EmailError> {
        let email = Message::builder()
            .from(self.smtp_config.from.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(html.to_string())
            )?;

        self.transport.send(&email).await?;
        log::info!("HTML email sent to {}", to);
        Ok(())
    }

    // 使用模板发送邮件
    pub async fn send_template(
        &self,
        to: &str,
        subject: &str,
        template_name: &str,
        variables: &serde_json::Value,
    ) -> Result<(), EmailError> {
        let html = self.template_engine.render(template_name, variables)?;
        self.send_html(to, subject, &html).await
    }

    // 发送带附件的邮件
    pub async fn send_with_attachment(
        &self,
        to: &str,
        subject: &str,
        text: &str,
        attachment_path: &Path,
    ) -> Result<(), EmailError> {
        let attachment_data = tokio::fs::read(attachment_path).await?;

        let email = Message::builder()
            .from(self.smtp_config.from.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .multipart(
                MultiPart::mixed()
                    .singlepart(
                        SinglePart::plain().body(text.to_string())
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::parse(
                                attachment_path.extension()
                                    .and_then(|e| e.to_str())
                                    .unwrap_or("application/octet-stream")
                            )?)
                            .header(header::ContentDisposition::attachment(
                                attachment_path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("attachment")
                            ))
                            .body(attachment_data)
                    )
            )?;

        self.transport.send(&email).await?;
        log::info!("Email with attachment sent to {}", to);
        Ok(())
    }

    // 批量发送
    pub async fn send_bulk(
        &self,
        recipients: Vec<String>,
        subject: &str,
        content: &str,
    ) -> Vec<Result<(), EmailError>> {
        let mut results = Vec::new();

        for to in recipients {
            let result = self.send(&to, subject, content).await;
            results.push(result);
        }

        results
    }
}

// 控制器使用
#[controller]
struct EmailController;

#[post("/api/email/send")]
async fn send_email(
    #[request_body] request: SendEmailRequest,
    #[state] email_service: Arc<EmailService>,
) -> Result<Status, Error> {
    email_service.send(
        &request.to,
        &request.subject,
        &request.content,
    ).await?;

    Ok(Status::OK)
}

#[derive(Deserialize)]
pub struct SendEmailRequest {
    pub to: String,
    pub subject: String,
    pub content: String,
}
```

---

## 第16章：跨域请求处理

### CORS 配置对比 / CORS Configuration Comparison

#### Spring Boot - CORS 配置

```java
// 1. 全局 CORS 配置
@Configuration
public class CorsConfig {

    @Bean
    public CorsConfigurationSource corsConfigurationSource() {
        CorsConfiguration configuration = new CorsConfiguration();

        // 允许的源
        configuration.setAllowedOriginPatterns(Arrays.asList("*"));

        // 允许的方法
        configuration.setAllowedMethods(Arrays.asList(
            "GET", "POST", "PUT", "DELETE", "OPTIONS", "PATCH"
        ));

        // 允许的请求头
        configuration.setAllowedHeaders(Arrays.asList("*"));

        // 允许发送凭证
        configuration.setAllowCredentials(true);

        // 预检请求缓存时间（秒）
        configuration.setMaxAge(3600L);

        // 暴露的响应头
        configuration.setExposedHeaders(Arrays.asList(
            "Authorization", "Content-Disposition", "X-Total-Count"
        ));

        UrlBasedCorsConfigurationSource source = new UrlBasedCorsConfigurationSource();
        source.registerCorsConfiguration("/**", configuration);
        return source;
    }
}

// 2. Controller 级别 CORS
@RestController
@RequestMapping("/api/users")
@CrossOrigin(
    origins = {"http://localhost:3000", "https://example.com"},
    methods = {RequestMethod.GET, RequestMethod.POST},
    allowedHeaders = {"Authorization", "Content-Type"},
    exposedHeaders = {"X-Total-Count"},
    allowCredentials = "true",
    maxAge = 3600
)
public class UserController {
    // ...
}

// 3. 方法级别 CORS
@GetMapping("/{id}")
@CrossOrigin(origins = "http://localhost:3000")
public User getUser(@PathVariable Long id) {
    return userService.findById(id);
}
```

#### Nexus - CORS 中间件

```rust
use hiver_middleware::{CorsMiddleware, CorsConfig};

// 1. 全局 CORS 配置
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cors_config = CorsConfig::new()
        // 允许的源
        .allowed_origins(vec![
            "http://localhost:3000".to_string(),
            "https://example.com".to_string(),
        ])
        // 允许的方法
        .allowed_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        // 允许的请求头
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
        ])
        // 允许凭证
        .allow_credentials(true)
        // 预检请求缓存时间
        .max_age(Duration::from_secs(3600))
        // 暴露的响应头
        .exposed_headers(vec![
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-disposition"),
            HeaderName::from_static("x-total-count"),
        ]);

    let app = Router::new()
        .get("/api/users", list_users)
        .middleware(Arc::new(CorsMiddleware::new(cors_config)));

    Server::bind("127.0.0.1:8080")
        .serve(app)
        .await?;

    Ok(())
}

// 2. 路由组级别 CORS
fn api_routes() -> Router {
    let cors_config = CorsConfig::new()
        .allowed_origin("http://localhost:3000")
        .allowed_methods(vec![Method::GET, Method::POST])
        .allow_credentials(true);

    Router::new()
        .get("/users", list_users)
        .post("/users", create_user)
        .middleware(Arc::new(CorsMiddleware::new(cors_config)))
}

// 3. 允许所有源（开发环境）
fn dev_cors_config() -> CorsConfig {
    CorsConfig::new()
        .allow_all()
        .allow_credentials(false) // allow_all 时不能启用凭证
}
```

---

## 第17章：前后端接口统一响应结构

### 统一响应对比 / Unified Response Comparison

#### Spring Boot - Result<T>

```java
// 1. 响应码枚举
@Getter
@AllArgsConstructor
public enum ResultCode {
    SUCCESS(200, "操作成功"),
    BAD_REQUEST(400, "请求参数错误"),
    UNAUTHORIZED(401, "未认证"),
    FORBIDDEN(403, "无权限"),
    NOT_FOUND(404, "资源不存在"),
    INTERNAL_ERROR(500, "服务器内部错误"),

    // 业务错误码
    USER_NOT_FOUND(1001, "用户不存在"),
    USER_ALREADY_EXISTS(1002, "用户已存在"),
    INVALID_PASSWORD(1003, "密码错误"),
    ;

    private final int code;
    private final String message;
}

// 2. 统一响应结构
@Data
@NoArgsConstructor
@AllArgsConstructor
public class Result<T> {
    private Integer code;
    private String message;
    private T data;
    private Long timestamp;
    private String path;

    public static <T> Result<T> success() {
        return success(null);
    }

    public static <T> Result<T> success(T data) {
        return new Result<>(
            ResultCode.SUCCESS.getCode(),
            ResultCode.SUCCESS.getMessage(),
            data,
            System.currentTimeMillis(),
            ""
        );
    }

    public static <T> Result<T> error(ResultCode resultCode) {
        return new Result<>(
            resultCode.getCode(),
            resultCode.getMessage(),
            null,
            System.currentTimeMillis(),
            ""
        );
    }

    public static <T> Result<T> error(Integer code, String message) {
        return new Result<>(
            code,
            message,
            null,
            System.currentTimeMillis(),
            ""
        );
    }

    public static <T> Result<T> error(ResultCode resultCode, String message) {
        return new Result<>(
            resultCode.getCode(),
            message,
            null,
            System.currentTimeMillis(),
            ""
        );
    }
}

// 3. 分页结果
@Data
@AllArgsConstructor
public class PageResult<T> {
    private List<T> content;
    private Integer page;
    private Integer size;
    private Long totalElements;
    private Integer totalPages;

    public static <T> PageResult<T> of(Page<T> page) {
        return new PageResult<>(
            page.getContent(),
            page.getNumber(),
            page.getSize(),
            page.getTotalElements(),
            page.getTotalPages()
        );
    }
}

// 4. 全局响应处理器
@RestControllerAdvice
public class ResponseAdvice implements ResponseBodyAdvice<Object> {

    @Override
    public boolean supports(
        MethodParameter returnType,
        Class converterType
    ) {
        // 不包装已经是 Result 类型的响应
        return !returnType.getParameterType().equals(Result.class);
    }

    @Override
    public Object beforeBodyWrite(
        Object body,
        MethodParameter returnType,
        MediaType selectedContentType,
        Class selectedConverterType,
        ServerHttpRequest request,
        ServerHttpResponse response
    ) {
        // String 类型特殊处理
        if (body instanceof String) {
            return JSON.toJSONString(Result.success(body));
        }

        return Result.success(body);
    }
}

// 5. 使用示例
@RestController
@RequestMapping("/api/users")
public class UserController {

    @GetMapping
    public Result<PageResult<User>> listUsers(
        @RequestParam(defaultValue = "0") int page,
        @RequestParam(defaultValue = "10") int size
    ) {
        Page<User> userPage = userService.findAll(page, size);
        return Result.success(PageResult.of(userPage));
    }

    @GetMapping("/{id}")
    public Result<User> getUser(@PathVariable Long id) {
        User user = userService.findById(id)
            .orElseThrow(() -> new NotFoundException("用户不存在"));
        return Result.success(user);
    }

    @PostMapping
    public Result<User> createUser(@RequestBody @Valid CreateUserRequest request) {
        User user = userService.create(request);
        return Result.success(user);
    }

    @ExceptionHandler(NotFoundException.class)
    public Result<Void> handleNotFound(NotFoundException e) {
        return Result.error(ResultCode.NOT_FOUND.getCode(), e.getMessage());
    }
}
```

#### Nexus - 统一响应结构

```rust
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// 1. 响应码枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultCode {
    Success = 200,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    InternalError = 500,

    // 业务错误码
    UserNotFound = 1001,
    UserAlreadyExists = 1002,
    InvalidPassword = 1003,
}

impl ResultCode {
    pub fn code(&self) -> u16 {
        *self as u16
    }

    pub fn message(&self) -> &'static str {
        match self {
            ResultCode::Success => "操作成功",
            ResultCode::BadRequest => "请求参数错误",
            ResultCode::Unauthorized => "未认证",
            ResultCode::Forbidden => "无权限",
            ResultCode::NotFound => "资源不存在",
            ResultCode::InternalError => "服务器内部错误",
            ResultCode::UserNotFound => "用户不存在",
            ResultCode::UserAlreadyExists => "用户已存在",
            ResultCode::InvalidPassword => "密码错误",
        }
    }
}

// 2. 统一响应结构
#[derive(Debug, Serialize)]
pub struct Result<T> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub errors: HashMap<String, String>,
}

impl<T> Result<T> {
    pub fn success() -> Self {
        Self::success_data(None)
    }

    pub fn success_data(data: T) -> Self {
        Self {
            code: ResultCode::Success.code(),
            message: ResultCode::Success.message().to_string(),
            data: Some(data),
            timestamp: Utc::now().timestamp(),
            path: None,
            errors: HashMap::new(),
        }
    }

    pub fn error(code: ResultCode) -> Self {
        Self::error_message(code, code.message())
    }

    pub fn error_message(code: ResultCode, message: &str) -> Self {
        Self {
            code: code.code(),
            message: message.to_string(),
            data: None,
            timestamp: Utc::now().timestamp(),
            path: None,
            errors: HashMap::new(),
        }
    }

    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_errors(mut self, errors: HashMap<String, String>) -> Self {
        self.errors = errors;
        self
    }
}

// 3. 分页结果
#[derive(Debug, Serialize)]
pub struct PageResult<T> {
    pub content: Vec<T>,
    pub page: u32,
    pub size: u32,
    pub total_elements: u64,
    pub total_pages: u32,
}

impl<T> PageResult<T> {
    pub fn new(
        content: Vec<T>,
        page: u32,
        size: u32,
        total_elements: u64,
    ) -> Self {
        let total_pages = if size == 0 {
            0
        } else {
            ((total_elements as f64) / (size as f64)).ceil() as u32
        };

        Self {
            content,
            page,
            size,
            total_elements,
            total_pages,
        }
    }

    pub fn empty() -> Self {
        Self::new(vec![], 0, 10, 0)
    }
}

// 4. IntoResponse for Result
impl<T: Serialize> IntoResponse for Result<T> {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

// 5. 控制器使用
#[controller]
struct UserController;

#[get("/api/users")]
async fn list_users(
    #[query] page: Option<u32>,
    #[query] size: Option<u32>,
    #[state] service: Arc<UserService>,
) -> Result<PageResult<User>, Error> {
    let page = page.unwrap_or(0);
    let size = size.unwrap_or(10);
    let users = service.find_all(page, size).await?;
    Ok(users)
}

#[get("/api/users/:id")]
async fn get_user(
    id: i64,
    #[state] service: Arc<UserService>,
) -> Result<Json<User>, Error> {
    let user = service.find_by_id(id).await?;
    Ok(Json(user))
}

#[post("/api/users")]
async fn create_user(
    #[request_body] request: CreateUserRequest,
    #[state] service: Arc<UserService>,
) -> Result<Json<User>, Error> {
    let user = service.create(request).await?;
    Ok(Json(user))
}
```

---

## 第18章：分页查询接口标准实现

### 分页实现对比 / Pagination Implementation Comparison

#### Spring Boot - 分页

```java
// 1. JPA 分页
@Repository
public interface UserRepository extends JpaRepository<User, Long> {

    // 基础分页查询
    Page<User> findAll(Pageable pageable);

    // 条件分页查询
    @Query("SELECT u FROM User u WHERE u.username LIKE %:keyword%")
    Page<User> findByUsernameContaining(@Param("keyword") String keyword, Pageable pageable);

    // 多条件分页
    @Query("SELECT u FROM User u WHERE " +
           "(:keyword IS NULL OR u.username LIKE %:keyword%) AND " +
           "(:status IS NULL OR u.status = :status)")
    Page<User> findByConditions(
        @Param("keyword") String keyword,
        @Param("status") UserStatus status,
        Pageable pageable
    );
}

// 2. Service 层
@Service
public class UserService {

    @Autowired
    private UserRepository userRepository;

    // 分页查询
    public PageResult<User> findAll(int page, int size) {
        Pageable pageable = PageRequest.of(page, size, Sort.by("id").descending());
        Page<User> userPage = userRepository.findAll(pageable);
        return PageResult.of(userPage);
    }

    // 搜索分页
    public PageResult<User> search(String keyword, int page, int size) {
        Pageable pageable = PageRequest.of(page, size);
        Page<User> userPage = userRepository.findByUsernameContaining(keyword, pageable);
        return PageResult.of(userPage);
    }

    // 条件查询分页
    public PageResult<User> findByConditions(UserQuery query, int page, int size) {
        Pageable pageable = PageRequest.of(
            page,
            size,
            Sort.by(
                Sort.Order.desc(query.getSortField()),
                Sort.Order.asc("id")
            )
        );
        Page<User> userPage = userRepository.findByConditions(
            query.getKeyword(),
            query.getStatus(),
            pageable
        );
        return PageResult.of(userPage);
    }
}

// 3. Controller 层
@RestController
@RequestMapping("/api/users")
public class UserController {

    @GetMapping
    public Result<PageResult<User>> listUsers(
        @RequestParam(defaultValue = "0") int page,
        @RequestParam(defaultValue = "10") int size,
        @RequestParam(defaultValue = "id") String sort,
        @RequestParam(required = false) String keyword
    ) {
        if (keyword != null && !keyword.isEmpty()) {
            return Result.success(userService.search(keyword, page, size));
        }
        return Result.success(userService.findAll(page, size));
    }

    @PostMapping("/search")
    public Result<PageResult<User>> searchUsers(
        @RequestBody UserQuery query,
        @RequestParam(defaultValue = "0") int page,
        @RequestParam(defaultValue = "10") int size
    ) {
        return Result.success(userService.findByConditions(query, page, size));
    }
}

// 4. 查询条件对象
@Data
public class UserQuery {
    private String keyword;
    private UserStatus status;
    private String sortField = "id";
    private String sortOrder = "DESC";
}
```

#### Nexus - 分页实现

```rust
use hiver::prelude::*;

// 1. 分页请求
#[derive(Debug, Deserialize)]
pub struct PageRequest {
    #[serde(default = "default_page")]
    pub page: u32,

    #[serde(default = "default_size")]
    pub size: u32,

    #[serde(default)]
    pub sort: String,

    #[serde(default = "default_sort_order")]
    pub sort_order: String,
}

fn default_page() -> u32 { 0 }
fn default_size() -> u32 { 10 }
fn default_sort_order() -> String { "desc".to_string() }

impl PageRequest {
    pub fn limit(&self) -> usize {
        self.size as usize
    }

    pub fn offset(&self) -> usize {
        self.page as usize * self.size as usize
    }

    pub fn validate(&self) -> Result<(), Error> {
        const MAX_SIZE: u32 = 100;
        if self.size > MAX_SIZE {
            return Err(Error::bad_request(&format!("每页数量不能超过 {}", MAX_SIZE)));
        }
        Ok(())
    }
}

// 2. Repository 层
#[repository]
pub trait UserRepository: Send + Sync {
    async fn find_page(&self, req: &PageRequest) -> Result<(Vec<User>, u64), DbError>;

    async fn search_page(&self, keyword: &str, req: &PageRequest) -> Result<(Vec<User>, u64), DbError>;

    async fn find_by_conditions_page(
        &self,
        conditions: &UserQuery,
        req: &PageRequest
    ) -> Result<(Vec<User>, u64), DbError>;
}

// PostgreSQL 实现
impl UserRepository for PostgresUserRepository {
    async fn find_page(&self, req: &PageRequest) -> Result<(Vec<User>, u64), DbError> {
        // 查询总数
        let total: i64 = self.db.query_one(
            "SELECT COUNT(*) FROM users"
        ).await?.get(0);

        // 查询数据
        let users = query_as::<_, User>(
            "SELECT * FROM users ORDER BY id $1 LIMIT $2 OFFSET $3"
        )
        .bind(sort_order(req.sort_order.as_str()))
        .bind(req.limit() as i64)
        .bind(req.offset() as i64)
        .fetch_all(&self.db)
        .await?;

        Ok((users, total as u64))
    }

    async fn search_page(&self, keyword: &str, req: &PageRequest) -> Result<(Vec<User>, u64), DbError> {
        let pattern = format!("%{}%", keyword);

        // 查询总数
        let total: i64 = self.db.query_one(
            "SELECT COUNT(*) FROM users WHERE username LIKE $1 OR email LIKE $1"
        )
        .bind(&pattern)
        .await?
        .get(0);

        // 查询数据
        let users = query_as::<_, User>(
            "SELECT * FROM users WHERE username LIKE $1 OR email LIKE $1 ORDER BY id $2 LIMIT $3 OFFSET $4"
        )
        .bind(&pattern)
        .bind(sort_order(req.sort_order.as_str()))
        .bind(req.limit() as i64)
        .bind(req.offset() as i64)
        .fetch_all(&self.db)
        .await?;

        Ok((users, total as u64))
    }

    async fn find_by_conditions_page(
        &self,
        conditions: &UserQuery,
        req: &PageRequest
    ) -> Result<(Vec<User>, u64), DbError> {
        let mut query = String::from("SELECT * FROM users WHERE 1=1");
        let mut count_query = String::from("SELECT COUNT(*) FROM users WHERE 1=1");
        let mut params = Vec::new();
        let mut param_idx = 1;

        if let Some(keyword) = &conditions.keyword {
            if !keyword.is_empty() {
                query.push_str(&format!(" AND (username LIKE ${} OR email LIKE ${})", param_idx, param_idx));
                count_query.push_str(&format!(" AND (username LIKE ${} OR email LIKE ${})", param_idx, param_idx));
                params.push(format!("%{}%", keyword));
                param_idx += 1;
            }
        }

        if let Some(status) = &conditions.status {
            query.push_str(&format!(" AND status = ${}", param_idx));
            count_query.push_str(&format!(" AND status = ${}", param_idx));
            params.push(status.to_string());
            param_idx += 1;
        }

        // 查询总数
        let total: i64 = self.db.query_one(&count_query).await?.get(0);

        // 添加排序和分页
        query.push_str(&format!(" ORDER BY id {}", sort_order(req.sort_order.as_str())));
        query.push_str(&format!(" LIMIT ${} OFFSET ${}", param_idx, param_idx + 1));

        // 查询数据
        let mut q = query_as::<_, User>(&query);
        for param in params {
            q = q.bind(param);
        }
        q = q.bind(req.limit() as i64);
        q = q.bind(req.offset() as i64);

        let users = q.fetch_all(&self.db).await?;

        Ok((users, total as u64))
    }
}

fn sort_order(order: &str) -> &'static str {
    match order.to_lowercase().as_str() {
        "asc" => "ASC",
        _ => "DESC",
    }
}

// 3. Service 层
#[service]
pub struct UserService {
    #[autowired]
    repository: Arc<dyn UserRepository>,
}

impl UserService {
    pub async fn find_all(&self, page: u32, size: u32) -> Result<PageResult<User>, Error> {
        let req = PageRequest { page, size, ..Default::default() };
        req.validate()?;

        let (users, total) = self.repository.find_page(&req).await?;
        Ok(PageResult::new(users, page, size, total))
    }

    pub async fn search(&self, keyword: &str, page: u32, size: u32) -> Result<PageResult<User>, Error> {
        let req = PageRequest { page, size, ..Default::default() };
        req.validate()?;

        let (users, total) = self.repository.search_page(keyword, &req).await?;
        Ok(PageResult::new(users, page, size, total))
    }

    pub async fn find_by_conditions(
        &self,
        query: UserQuery,
        page: u32,
        size: u32
    ) -> Result<PageResult<User>, Error> {
        let req = PageRequest {
            page,
            size,
            sort: query.sort_field.clone(),
            sort_order: query.sort_order.clone(),
        };
        req.validate()?;

        let (users, total) = self.repository.find_by_conditions_page(&query, &req).await?;
        Ok(PageResult::new(users, page, size, total))
    }
}

// 4. Controller 层
#[controller]
struct UserController;

#[get("/api/users")]
async fn list_users(
    #[query] page: Option<u32>,
    #[query] size: Option<u32>,
    #[query] sort: Option<String>,
    #[query] keyword: Option<String>,
    #[state] service: Arc<UserService>,
) -> Result<Json<PageResult<User>>, Error> {
    let page = page.unwrap_or(0);
    let size = size.unwrap_or(10);

    if let Some(k) = keyword {
        if !k.is_empty() {
            return Ok(Json(service.search(&k, page, size).await?));
        }
    }

    Ok(Json(service.find_all(page, size).await?))
}

#[post("/api/users/search")]
async fn search_users(
    #[request_body] query: UserQuery,
    #[query] page: Option<u32>,
    #[query] size: Option<u32>,
    #[state] service: Arc<UserService>,
) -> Result<Json<PageResult<User>>, Error> {
    let page = page.unwrap_or(0);
    let size = size.unwrap_or(10);

    Ok(Json(service.find_by_conditions(query, page, size).await?))
}

// 5. 查询条件对象
#[derive(Debug, Deserialize)]
pub struct UserQuery {
    pub keyword: Option<String>,
    pub status: Option<String>,
    #[serde(default = "default_sort_field")]
    pub sort_field: String,
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
}

fn default_sort_field() -> String { "id".to_string() }

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: 0,
            size: 10,
            sort: "id".to_string(),
            sort_order: "desc".to_string(),
        }
    }
}
```

---

## 第19章：文件导出与报表生成

### 文件导出对比 / File Export Comparison

#### Spring Boot - 文件导出

```java
// 1. CSV 导出
@Service
public class CsvExportService {

    public void exportUsersToCsv(HttpServletResponse response, List<User> users) throws IOException {
        response.setContentType("text/csv");
        response.setHeader("Content-Disposition", "attachment; filename=users.csv");

        try (PrintWriter writer = response.getWriter()) {
            // CSV 头部
            writer.println("ID,Username,Email,Created At");

            // 数据行
            for (User user : users) {
                writer.printf("%d,%s,%s,%s%n",
                    user.getId(),
                    escapeCsv(user.getUsername()),
                    escapeCsv(user.getEmail()),
                    user.getCreatedAt()
                );
            }
        }
    }

    private String escapeCsv(String value) {
        if (value.contains(",") || value.contains("\"") || value.contains("\n")) {
            return "\"" + value.replace("\"", "\"\"") + "\"";
        }
        return value;
    }
}

// 2. Excel 导出 (使用 Apache POI)
@Service
public class ExcelExportService {

    public void exportUsersToExcel(HttpServletResponse response, List<User> users) throws IOException {
        Workbook workbook = new XSSFWorkbook();
        Sheet sheet = workbook.createSheet("Users");

        // 创建头部样式
        CellStyle headerStyle = workbook.createCellStyle();
        Font headerFont = workbook.createFont();
        headerFont.setBold(true);
        headerStyle.setFont(headerFont);
        headerStyle.setFillForegroundColor(IndexedColors.GREY_25_PERCENT.getIndex());
        headerStyle.setFillPattern(FillPatternType.SOLID_FOREGROUND);

        // 创建头部
        Row headerRow = sheet.createRow(0);
        String[] headers = {"ID", "Username", "Email", "Created At"};
        for (int i = 0; i < headers.length; i++) {
            Cell cell = headerRow.createCell(i);
            cell.setCellValue(headers[i]);
            cell.setCellStyle(headerStyle);
        }

        // 填充数据
        int rowNum = 1;
        for (User user : users) {
            Row row = sheet.createRow(rowNum++);
            row.createCell(0).setCellValue(user.getId());
            row.createCell(1).setCellValue(user.getUsername());
            row.createCell(2).setCellValue(user.getEmail());
            row.createCell(3).setCellValue(user.getCreatedAt().toString());
        }

        // 自动调整列宽
        for (int i = 0; i < headers.length; i++) {
            sheet.autoSizeColumn(i);
        }

        // 输出
        response.setContentType("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
        response.setHeader("Content-Disposition", "attachment; filename=users.xlsx");

        workbook.write(response.getOutputStream());
        workbook.close();
    }
}

// 3. PDF 导出 (使用 iText)
@Service
public class PdfExportService {

    public void exportUsersToPdf(HttpServletResponse response, List<User> users) throws IOException {
        response.setContentType("application/pdf");
        response.setHeader("Content-Disposition", "attachment; filename=users.pdf");

        try (PdfWriter writer = new PdfWriter(response.getOutputStream());
             PdfDocument pdf = new PdfDocument(writer);
             Document document = new Document(pdf, PageSize.A4())) {

            // 添加标题
            PdfFont font = PdfFontFactory.createFont(StandardFonts.HELVETICA_BOLD);
            document.add(new Paragraph("User List").setFont(font).setFontSize(18));

            // 创建表格
            Table table = new Table(UnitValue.createPercentArray(new float[]{1, 3, 3, 3}));
            table.setWidth(UnitValue.createPercentValue(100));

            // 表头
            table.addHeaderCell(new Cell().add(new Paragraph("ID").setFont(font).setBackgroundColor(ColorConstants.LIGHT_GRAY)));
            table.addHeaderCell(new Cell().add(new Paragraph("Username").setFont(font).setBackgroundColor(ColorConstants.LIGHT_GRAY)));
            table.addHeaderCell(new Cell().add(new Paragraph("Email").setFont(font).setBackgroundColor(ColorConstants.LIGHT_GRAY)));
            table.addHeaderCell(new Cell().add(new Paragraph("Created At").setFont(font).setBackgroundColor(ColorConstants.LIGHT_GRAY)));

            // 数据行
            for (User user : users) {
                table.addCell(new Cell().add(new Paragraph(String.valueOf(user.getId()))));
                table.addCell(new Cell().add(new Paragraph(user.getUsername())));
                table.addCell(new Cell().add(new Paragraph(user.getEmail())));
                table.addCell(new Cell().add(new Paragraph(user.getCreatedAt().toString())));
            }

            document.add(table);
        } catch (Exception e) {
            throw new IOException("Failed to generate PDF", e);
        }
    }
}

// 4. Controller
@RestController
@RequestMapping("/api/export")
public class ExportController {

    @Autowired
    private CsvExportService csvExportService;

    @Autowired
    private ExcelExportService excelExportService;

    @Autowired
    private PdfExportService pdfExportService;

    @Autowired
    private UserService userService;

    @GetMapping("/users/csv")
    public void exportUsersCsv(HttpServletResponse response) throws IOException {
        List<User> users = userService.findAll();
        csvExportService.exportUsersToCsv(response, users);
    }

    @GetMapping("/users/excel")
    public void exportUsersExcel(HttpServletResponse response) throws IOException {
        List<User> users = userService.findAll();
        excelExportService.exportUsersToExcel(response, users);
    }

    @GetMapping("/users/pdf")
    public void exportUsersPdf(HttpServletResponse response) throws IOException {
        List<User> users = userService.findAll();
        pdfExportService.exportUsersToPdf(response, users);
    }
}
```

#### Nexus - 文件导出

```rust
use hiver::prelude::*;
use csv::Writer;
use calamine::{Writer as XlsxWriter, Workbook};
use std::io::Cursor;

// 1. CSV 导出服务
#[service]
pub struct CsvExportService;

impl CsvExportService {
    pub async fn export_users(&self, users: Vec<User>) -> Result<Vec<u8>, ExportError> {
        let mut writer = Writer::from_writer(Vec::new())?;

        // 写入头部
        writer.write_record(&["ID", "Username", "Email", "Created At"])?;

        // 写入数据
        for user in users {
            writer.write_record(&[
                user.id.to_string(),
                self.escape_csv(&user.username),
                self.escape_csv(&user.email),
                user.created_at.to_rfc3339(),
            ])?;
        }

        Ok(writer.into_inner()?)
    }

    fn escape_csv(&self, value: &str) -> String {
        if value.contains(',') || value.contains('"') || value.contains('\n') {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_string()
        }
    }
}

// 2. Excel 导出服务
#[service]
pub struct ExcelExportService;

impl ExcelExportService {
    pub async fn export_users(&self, users: Vec<User>) -> Result<Vec<u8>, ExportError> {
        let mut workbook = XlsxWriter::new();

        // 创建工作表
        let sheet = workbook.create_sheet("Users");

        // 写入头部
        workbook.write_sheet(&sheet, &[
            vec!["ID".to_string(), "Username".to_string(), "Email".to_string(), "Created At".to_string()]
        ])?;

        // 写入数据
        let mut rows = vec![];
        for user in users {
            rows.push(vec![
                user.id.to_string(),
                user.username.clone(),
                user.email.clone(),
                user.created_at.to_rfc3339(),
            ]);
        }
        workbook.write_sheet(&sheet, &rows)?;

        // 生成字节
        let mut buffer = vec![];
        workbook.save_to(&mut buffer)?;

        Ok(buffer)
    }
}

// 3. PDF 导出服务
#[service]
pub struct PdfExportService;

impl PdfExportService {
    pub async fn export_users(&self, users: Vec<User>) -> Result<Vec<u8>, ExportError> {
        use printpdf::{Pt, Mm, PdfDocument};

        let (doc, page1, layer1) = PdfDocument::new("User List", Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // 添加标题
        let font = doc.add_builtin_font(printpdf::BuiltinFont::HelveticaBold).unwrap();
        // ... PDF 绘制逻辑

        let mut buffer = vec![];
        doc.save(&mut std::io::Cursor::new(&mut buffer))?;

        Ok(buffer)
    }
}

// 4. Controller
#[controller]
struct ExportController;

#[get("/api/export/users/csv")]
async fn export_users_csv(
    #[query] format: Option<String>,
    #[state] user_service: Arc<UserService>,
    #[state] csv_service: Arc<CsvExportService>,
) -> Result<Response, Error> {
    let users = user_service.find_all().await;
    let data = csv_service.export_users(users).await?;

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "text/csv; charset=utf-8")
        .header("Content-Disposition", "attachment; filename=users.csv")
        .header("Content-Length", data.len())
        .body(Body::from(data))
        .map_err(|e| Error::internal(&e.to_string()))?)
}

#[get("/api/export/users/excel")]
async fn export_users_excel(
    #[state] user_service: Arc<UserService>,
    #[state] excel_service: Arc<ExcelExportService>,
) -> Result<Response, Error> {
    let users = user_service.find_all().await;
    let data = excel_service.export_users(users).await?;

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .header("Content-Disposition", "attachment; filename=users.xlsx")
        .header("Content-Length", data.len())
        .body(Body::from(data))
        .map_err(|e| Error::internal(&e.to_string()))?)
}

#[get("/api/export/users/pdf")]
async fn export_users_pdf(
    #[state] user_service: Arc<UserService>,
    #[state] pdf_service: Arc<PdfExportService>,
) -> Result<Response, Error> {
    let users = user_service.find_all().await;
    let data = pdf_service.export_users(users).await?;

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/pdf")
        .header("Content-Disposition", "attachment; filename=users.pdf")
        .header("Content-Length", data.len())
        .body(Body::from(data))
        .map_err(|e| Error::internal(&e.to_string()))?)
}

// 5. 通用导出处理器
pub enum ExportFormat {
    Csv,
    Excel,
    Pdf,
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Some(ExportFormat::Csv),
            "excel" | "xlsx" => Some(ExportFormat::Excel),
            "pdf" => Some(ExportFormat::Pdf),
            _ => None,
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "text/csv",
            ExportFormat::Excel => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ExportFormat::Pdf => "application/pdf",
        }
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "csv",
            ExportFormat::Excel => "xlsx",
            ExportFormat::Pdf => "pdf",
        }
    }
}

#[get("/api/export/users")]
async fn export_users(
    #[query] format: String,
    #[state] user_service: Arc<UserService>,
    #[state] csv_service: Arc<CsvExportService>,
    #[state] excel_service: Arc<ExcelExportService>,
    #[state] pdf_service: Arc<PdfExportService>,
) -> Result<Response, Error> {
    let users = user_service.find_all().await;
    let export_format = ExportFormat::from_str(&format)
        .ok_or_else(|| Error::bad_request("不支持的导出格式"))?;

    let data = match export_format {
        ExportFormat::Csv => csv_service.export_users(users).await?,
        ExportFormat::Excel => excel_service.export_users(users).await?,
        ExportFormat::Pdf => pdf_service.export_users(users).await?,
    };

    let filename = format!("users.{}", export_format.file_extension());

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", export_format.content_type())
        .header("Content-Disposition", format!("attachment; filename=\"{}\"", filename))
        .header("Content-Length", data.len())
        .body(Body::from(data))
        .map_err(|e| Error::internal(&e.to_string()))?)
}
```

---

## 功能对比总结 / Summary

### 实战功能对比 / Practice Features Comparison

| 功能 / Feature | Spring Boot | Nexus | 完成度 |
|----------------|-------------|-------|--------|
| **文件上传 / File Upload** | MultipartFile | multer | ⚠️ 70% |
| **文件下载 / File Download** | ResponseEntity | Response::builder | ✅ 90% |
| **定时任务 / Scheduled** | @Scheduled | #[scheduled] | ✅ 85% |
| **异步执行 / Async** | @Async | tokio::spawn | ✅ 100% |
| **邮件发送 / Email** | JavaMailSender | lettre | ⚠️ 75% |
| **CORS / CORS** | @CrossOrigin | CorsMiddleware | ✅ 100% |
| **统一响应 / Result** | Result<T> | Result<T> | ⚠️ 80% |
| **分页查询 / Pagination** | Pageable | PageRequest | ✅ 90% |
| **CSV 导出 / CSV Export** | OpenCSV | csv crate | ✅ 85% |
| **Excel 导出 / Excel** | Apache POI | rust_xlsxwriter | ⚠️ 60% |
| **PDF 导出 / PDF** | iText | printpdf | ⚠️ 50% |

### 待补充功能 / Features to Add

1. **完善文件处理**
   - 添加文件类型验证
   - 实现文件分片上传
   - 支持大文件流式处理

2. **增强邮件服务**
   - 支持邮件模板
   - 实现邮件队列
   - 添加邮件发送追踪

3. **完善导出功能**
   - 增强 Excel 样式支持
   - 改进 PDF 生成质量
   - 支持更多导出格式
